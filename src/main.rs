use gio::prelude::*;
use gtk::prelude::*;
use gtk::ApplicationWindow;

use std::env::args;

use gdk::Screen;

use url::Url;
use serde_derive::Deserialize;

use std::collections::HashMap;

// This `derive` requires the `serde` dependency.
#[derive(Deserialize,Debug,Clone)]
pub struct Query {
    pub pages: HashMap<String, Page>
}

#[derive(Deserialize,Debug,Clone)]
pub struct Page {
    pub pageid: u64,
    pub title: String,
    #[serde(rename = "extract")]
    pub desc: Option<String>,
}
#[derive(Deserialize,Debug)]
struct Wiki {
    query: Query,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let application = gtk::Application::new(
        Some("org.main_window"),
        gio::ApplicationFlags::empty()
    )
    .expect("Initialization failed...");

    let height = Screen::height();
    let style = format!("#label1 {{
                            color: white;
                            font-size: {0}px;
                            text-shadow:  -{1}px -{1}px 0 black,
                                           {1}px -{1}px 0 black,
                                          -{1}px  {1}px 0 black,
                                           {1}px  {1}px 0 black;
                        }}", height/17, height/500);

    let clipboard = gtk::Clipboard::get(&gdk::SELECTION_CLIPBOARD);
    let t = clipboard.wait_for_text().unwrap();
    println!("t {}", t);

    // Extracting Etymological Information from Wiktionary --
    // https://stackoverflow.com/questions/52351081
    let url = Url::parse_with_params("https://fr.wiktionary.org/w/api.php?format=json&action=query&prop=extracts&explaintext&exlimit=1",
                                     &[("titles", t)])?;
    let resp = reqwest::get(url)
        .await?
        .json::<Wiki>()
        .await?;
    let mut text = String::new();
    if let Some((_, Page { desc, .. })) = resp.query.pages.iter().next() {
        if let Some(desc) = desc {
            let wik = desc.split(|f| f == '\n')
                                 .filter(|x| !x.is_empty())
                                 .collect::<Vec<_>>();
            let etym = wik.iter().skip_while(|&x| x == &"== Français ==")
                                 .skip_while(|&x| x == &"=== Étymologie ===")
                          .next();
            if let Some(etym) = etym {
                text.push_str(etym);
            }
        }
    }
    application.connect_startup(move |app| {
        // The CSS "magic" happens here.
        let provider = gtk::CssProvider::new();
        provider
            .load_from_data(style.as_bytes())
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
        build_ui(app, &text);
    });

    application.run(&args().collect::<Vec<_>>());

    Ok(())
}

fn build_ui(application: &gtk::Application, text: &str) {
    let window = ApplicationWindow::new(application);
    set_visual(&window, None);

    window.connect_screen_changed(set_visual);
    window.connect_draw(draw);

    window.set_title("Wiktitrage");
    window.set_app_paintable(true); // crucial for transparency

    // The container container.
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);

    let label = gtk::Label::new(Some(text));
    gtk::WidgetExt::set_widget_name(&label, "label1");

    vbox.add(&label);
    // Then we add the container inside our window.
    window.add(&vbox);

    let w = window.clone();
    application.connect_activate(move |_| {
        w.show_all();
    });

    let mut counter: usize = 5;
    // we are using a closure to capture the label (else we could also use a normal function)
    let tick = move || {
        if let  Some(r) = counter.checked_sub(1) {
            counter = r;
            glib::Continue(true)
        } else {
            window.close();
            glib::Continue(false)
        }
    };

    // executes the closure once every second
    gtk::timeout_add_seconds(1, tick);
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
