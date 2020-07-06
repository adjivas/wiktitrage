#![crate_type = "bin"]
#![feature(get_mut_unchecked)]

use std::env;
use std::rc::Rc;

use getopts::Options;

use gio::prelude::*;
use gtk::prelude::*;
use gtk::ApplicationWindow;
use gtk::Justification;
use gdk::keys::constants;
use glib::clone;

pub mod style;
pub mod request;
pub use request::{Resp, Wik};

static APPLICATION_TIMER: usize = 7;
static APPLICATION_NAME: &str = "fr.adjivas.WikTitrage";

fn build_ui(application: &gtk::Application, wiks: Vec<Wik>) {
    let window = ApplicationWindow::new(application);
    set_visual(&window, None);

    let label = gtk::Label::new(Some(&wiks.first().unwrap().desc));
    let it_wik = wiks.into_iter().cycle();
    let rc_wik = Rc::new(it_wik);

    let timer: usize = APPLICATION_TIMER;
    let rc_timer = Rc::new(timer);


    // The container.
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);

    let screen = window.get_screen().unwrap();
    let geometry = screen.get_monitor_geometry(0);

    window.connect_screen_changed(set_visual);
    window.connect_draw(draw);

    window.set_default_size(geometry.width, -1);
    window.move_(0, geometry.height - 200);
    window.set_title("Wiktitrage");
    window.set_app_paintable(true); // crucial for transparency

    // Do not show in application switcher.
    window.set_skip_pager_hint(true);
    window.set_skip_taskbar_hint(true);

    // remove decoration
    window.set_resizable(false);
    window.set_decorated(false);

    window.connect_button_release_event(|win, _event| {
        win.close();
        Inhibit(false)
    });

    let r_time = rc_timer.clone();
    window.connect_key_press_event(clone!(@weak label =>
                                          @default-return Inhibit(false),
                                            move |win, event| {
        match event.get_keyval(){
            constants::Return => {
                let mut r_time = r_time.clone();
                let mut r_wik = rc_wik.clone();
                let it_wik = unsafe { Rc::get_mut_unchecked(&mut r_wik) };
                let time = unsafe { Rc::get_mut_unchecked(&mut r_time) };
                *time = APPLICATION_TIMER;
                if let Some (Wik { desc: text, .. }) = it_wik.next() {
                    let clip = gtk::Clipboard::get(&gdk::SELECTION_CLIPBOARD);
                    clip.set_text(&text);
                    label.set_text(&text);
                }
                Inhibit(true)
            }
            _ => {
                win.close();
                Inhibit(false)
            }
        }
    }));

    application.connect_activate(clone!(@weak window => move |_| {
        label.set_line_wrap(true);
        label.set_lines(3);
        label.set_justify(Justification::Center);
        gtk::WidgetExt::set_widget_name(&label, "label1");
        vbox.add(&label);
        // Then we add the container inside our window.
        window.add(&vbox);
        window.show_all();
    }));

    // executes the closure once every second
    gtk::timeout_add_seconds(1, move || {
        let mut r_time = rc_timer.clone();
        let time = unsafe {
            Rc::get_mut_unchecked(&mut r_time)
        };
        if let Some(timed) = time.checked_sub(1) {
            *time = timed;
            glib::Continue(true)
        } else {
            window.close();
            glib::Continue(false)
        }
    });

}

fn set_visual(window: &ApplicationWindow, _screen: Option<&gdk::Screen>) {
    if let Some(screen) = window.get_screen() {
        if let Some(ref visual) = screen.get_rgba_visual() {
            window.set_visual(Some(visual)); // crucial for transparency
        }
    }
}

fn draw(_window: &ApplicationWindow, ctx: &cairo::Context) -> Inhibit {
    // crucial for transparency
    ctx.set_source_rgba(0.0, 0.0, 0.0, 0.0);
    ctx.set_operator(cairo::Operator::Screen);
    ctx.paint();
    Inhibit(false)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("b", "height", "bottom-top subtitle distance", "HEIGHT");
    opts.optflag("v", "version", "print the version");
    opts.optflag("h", "help", "print this help menu");

    match opts.parse(&args[1..]) {
        Err(f) => { panic!(f.to_string()) }
        Ok(m) if m.opt_present("h") => {
            let brief = format!("Usage: {} [-v]", program);
            print!("{}", opts.usage(&brief));
        }
        Ok(m) if m.opt_present("v") => {
            println!("{} {}", env!("CARGO_PKG_NAME"),
                              env!("CARGO_PKG_VERSION"));
        }
        // TODO: add default arg b/height
        Ok(_) => {
            let application = gtk::Application::new(
                Some(APPLICATION_NAME),
                gio::ApplicationFlags::empty()
            )
            .expect("Initialization failed...");

            // TODO: wait gtk init
            let clipboard = gtk::Clipboard::get(&gdk::SELECTION_CLIPBOARD);
            let text = clipboard.wait_for_text().expect("Clip wait for text failed");
            let wiks = Resp::new(&text).await?.next().unwrap();

            clipboard.set_text(&wiks.first().unwrap().desc);

            // Extracting Etymological Information from Wiktionary --
            // https://stackoverflow.com/questions/52351081
            application.connect_startup(move |app| {
                // The CSS "magic" happens here.
                let css = style::get_style()
                    .unwrap_or_else(|_| String::from(style::APPLICATION_STYLE));
                let provider = gtk::CssProvider::new();

                provider
                    .load_from_data(css.as_bytes())
                    .expect("Failed to load CSS");
                // We give the CssProvided to the default screen so the CSS rules we added
                // can be applied to our window.
                gtk::StyleContext::add_provider_for_screen(
                    &gdk::Screen::get_default()
                    .expect("Error initializing gtk css provider."),
                    &provider,
                    gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
                    );

                // We build the application UI.
                build_ui(app, wiks.clone());
            });

            glib::set_application_name("wiktitrage");
            application.run(&args);
        },
    }
    Ok(())
}
