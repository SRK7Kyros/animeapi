use anyhow::{Ok as AHOk, Result as AHResult};
use fantoccini::{Client, ClientBuilder};
use serde::ser::StdError;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fmt;

pub async fn get_client() -> AHResult<Client> {
    let mut client = ClientBuilder::native();
    let client = client
        .capabilities(
            json!({
                "moz:firefoxOptions": {
                    "args":["-headless"]
                }
            })
            .as_object()
            .unwrap()
            .to_owned(),
        )
        .connect("http://127.0.0.1:4444/")
        .await?;
    Ok(client)
}

#[derive(Clone, Deserialize, Serialize, fmt::Debug)]
pub struct Anime {
    pub name: String,
    pub link: String,
    pub link_type: String,
    pub total_episodes: usize,
    pub available_episodes: usize,
    pub image_path: String,
}

impl Default for Anime {
    fn default() -> Anime {
        Anime {
            name: String::from(""),
            link: String::from(""),
            link_type: String::from(""),
            total_episodes: 0,
            available_episodes: 0,
            image_path: String::from(""),
        }
    }
}

pub trait AnimeStuff {
    fn from_json(json: &Value) -> Anime;
    fn to_json(&self) -> Value;
}

impl AnimeStuff for Anime {
    fn from_json(json: &Value) -> Anime {
        let object = json.as_object().unwrap();
        let key = object.keys().last().unwrap();

        let output = Anime {
            name: key.to_string(),
            link: json[key]["link"].to_string(),
            link_type: json[key]["link_type"].to_string(),
            total_episodes: json[key]["total_episodes"].as_u64().unwrap() as usize,
            available_episodes: json[key]["available_episodes"].as_u64().unwrap() as usize,
            image_path: json[key]["image_path"].to_string(),
        };

        output
    }
    fn to_json(&self) -> Value {
        let output = json!({
            * self.name: {
            "link": self.link,
            "link_type": self.link_type,
            "total_episodes": self.total_episodes,
            "available_episodes": self.available_episodes,
            "image_path": self.image_path,
        }});

        return output;
    }
}

pub mod animeunity {
    use crate::Anime;
    use anyhow::{Ok, Result as AHResult};
    use core::time;
    use fantoccini::ClientBuilder;
    use hyper::{Request, Uri};
    use std::{process::Output, thread, vec};
    use tokio::net::TcpStream;

    const LINK: &str = "https://www.animeunity.tv/";

    pub async fn search(term: String) -> AHResult<Vec<Anime>> {
        let output: Vec<Anime> = vec![];
        Ok(output)
    }

    async fn start_geckodriver() -> AHResult<Output> {
        Ok(std::process::Command::new("/Users/giulio/Desktop/geckodriver &        ").output()?)
    }

    pub async fn get_token() -> AHResult<String> {
        let output = start_geckodriver().await?;
        println!("status: {}", output.status);

        let client = ClientBuilder::native()
            .connect("https://localhost:4444")
            .await?;

        client
            .goto("https://www.animeunity.tv/anime/1469-naruto")
            .await?;

        thread::sleep(time::Duration::from_secs(5));

        let requests = client
            .execute("return window.performance.getEntries();", vec![])
            .await?;

        println!("{}", requests);

        Ok("".to_string())
    }
}
