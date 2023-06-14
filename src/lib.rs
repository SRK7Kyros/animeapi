use anyhow::{Error as AHError, Ok as AHOk, Result as AHResult};
use hyper::{body::HttpBody, Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fmt;
use thirtyfour::prelude::*;
use tokio::{
    io::{stdout, AsyncWriteExt},
    net::TcpStream,
    spawn,
};

pub async fn get_driver(headless: bool) -> AHResult<WebDriver> {
    let mut capabilities = DesiredCapabilities::firefox();
    if headless {
        capabilities.add_firefox_arg("-headless")?;
    }
    let driver = WebDriver::new("http://127.0.0.1:4444/session", capabilities).await?;
    Ok(driver)
}

pub async fn start_geckodriver() -> AHResult<()> {
    let sender = Client::new();

    let url = "http://127.0.0.1:4444/status".parse::<hyper::Uri>()?;

    let mut res = sender.get(url).await;
    if res.is_err() {
        spawn(async move {
            std::process::Command::new("/Users/giulio/Desktop/geckodriver").output();
        });
    } else {
        while let Some(chunk) = res.as_mut().unwrap().body_mut().data().await {
            stdout().write_all(&chunk?).await?;
        }
    }
    Ok(())
}

pub async fn stop_geckodriver(driver: Option<WebDriver>) -> AHResult<()> {
    let driver = match driver {
        Some(e) => e,
        None => {
            let driver = get_driver(true).await?;
            driver
        }
    };
    driver.quit().await?;
    println!("Successfully closed");
    Ok(())
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
    use hyper::{Request, Uri};
    use std::{process::Output, thread, vec};
    use thirtyfour::prelude::*;
    use tokio::net::TcpStream;

    const LINK: &str = "https://www.animeunity.tv/";

    pub async fn search(term: String) -> AHResult<Vec<Anime>> {
        let output: Vec<Anime> = vec![];
        Ok(output)
    }

    pub async fn get_token(headless: bool) -> AHResult<String> {
        let _ = crate::start_geckodriver().await;

        let driver = crate::get_driver(headless).await?;

        driver
            .goto("https://www.animeunity.tv/anime/1469-naruto")
            .await?;

        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

        let requests = driver
            .execute("return window.performance.getEntries();", vec![])
            .await?;

        let requests = serde_json::to_string_pretty(requests.json())?;

        driver.quit().await?;
        Ok(requests)
    }
}
