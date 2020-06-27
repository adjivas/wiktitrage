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
    // Extracting Etymological Information from Wiktionary --
    // https://stackoverflow.com/questions/52351081
    let url = Url::parse_with_params("https://fr.wiktionary.org/w/api.php?format=json&action=query&prop=extracts&explaintext&exlimit=1",
                                     &[("titles", "bonjour")])?;
    let resp = reqwest::get(url)
        .await?
        .json::<Wiki>()
        .await?;
    println!("{:?}", resp.query.pages.iter().next());
    Ok(())
}
