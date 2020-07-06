use std::io::prelude::*;
use std::io;
use std::fs::File;
use std::path::Path;
use std::env;

static APPLICATION_STYLE_FILES: [&str; 2] = [
    ".config/wiktitrage/config.css",
    "/etc/wiktitrage/config.css"
];
pub static APPLICATION_STYLE: &str = "
    #label1 {
       color: white;
       font-size: 2rem;
       text-shadow:  -2.5px -2.5px 0 black,
                     -2.5px -2.5px 0 black,
                     -2.5px  2.5px 0 black,
                      2.5px  2.5px 0 black;
    }
";

pub fn get_style() -> io::Result<String> {
    let mut buffer = String::new();

    let mut f = File::open(Path::new(&env::var("HOME").unwrap_or_default())
                                .join(APPLICATION_STYLE_FILES[0]))
                     .or_else(|_| File::open(APPLICATION_STYLE_FILES[1]))?;

    f.read_to_string(&mut buffer)?;
    Ok(buffer)
}
