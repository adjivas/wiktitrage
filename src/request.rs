use std::collections::hash_map::IntoIter;
use std::collections::HashMap;

use serde_derive::Deserialize;
use url::Url;

static APP_USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
);

// This `derive` requires the `serde` dependency.
#[derive(Deserialize,Debug,Clone)]
struct Query {
    pub pages: HashMap<String, Page>
}

#[derive(Debug,Clone)]
pub struct Wik {
    pub desc: String,
    lang: String,
}

#[derive(Deserialize,Debug,Clone)]
struct Page {
    pub pageid: u64,
    pub title: String,
    #[serde(rename = "extract")]
    pub desc: Option<String>,
}

impl Page {
    pub fn get_line(&self) -> Option<Vec<Wik>> {
       let desc: &String = self.desc.as_ref()?;
       Some(desc.split(|f| f == '\n')
           .filter(|e| !e.is_empty())
           .collect::<Vec<_>>()
           .as_slice()
           .windows(3)
           .filter_map(|elf| {
               match elf {
                   [lang, "=== Étymologie ===", desc]
                       if lang.starts_with("== ")
                       && lang.ends_with(" ==")
                       && !desc.starts_with("Étymologie manquante ou incomplète") =>
                           Some(Wik {
                                desc: desc.to_string(),
                                lang: lang.to_string()
                          }),
                   _ => None,
               }
           })
       .collect::<Vec<Wik>>())
    }
}

#[derive(Deserialize,Debug)]
struct Wiki {
    query: Query,
}

pub struct Resp {
    it: IntoIter<String, Page>,
    
}

impl Resp {
    pub async fn new(word: &str) -> Result<Resp, reqwest::Error> {
        // Extracting Etymological Information from Wiktionary --
        // https://stackoverflow.com/questions/52351081
        let url = Url::parse_with_params(
           concat!("https://", "fr", ".wiktionary.org/w/api.php?&explaintext"),
               &[("format", "json"), ("action", "query"), ("prop", "extracts"),
                 ("exlimit", "1"), ("titles", word)]
        ).expect("url parse word");

        let client = reqwest::Client::builder()
            .user_agent(APP_USER_AGENT)
            .build()?;
        let resp: Wiki = client.get(url).send()
            .await?
            .json::<Wiki>()
            .await?;
        Ok(Resp {
            it: resp.query.pages.into_iter(),
        })
    }
}

impl Iterator for Resp {
    type Item = Vec<Wik>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((_, page)) = self.it.next() {
            page.get_line()
        } else {
            None
        }
    }
}
