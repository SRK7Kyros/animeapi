use anyhow::{Error as AHError, Ok as AHOk, Result as AHResult};
use hyper::{body::HttpBody, Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fmt;
use thirtyfour::prelude::*;
use tokio::{
    io::{stdout, AsyncWriteExt},
    net::TcpStream,
    process::Child,
    spawn,
};

pub async fn get_driver(headless: bool) -> AHResult<WebDriver> {
    let mut capabilities = DesiredCapabilities::firefox();
    if headless {
        capabilities.add_firefox_arg("-headless")?;
    }
    let driver = WebDriver::new("http://127.0.0.1:4444/session", capabilities)
        .await
        .unwrap();
    Ok(driver)
}

pub async fn start_geckodriver() -> AHResult<Child> {
    let robo = tokio::process::Command::new("/Users/giulio/Desktop/geckodriver").spawn()?;
    return Ok(robo);
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
    use hyper::{body::HttpBody, Body, Client, Method, Request};
    use hyper_tls::HttpsConnector;
    use serde_json::json;
    use std::{fmt::format, process::Output, thread, vec};
    use thirtyfour::prelude::*;
    use tokio::net::TcpStream;

    pub async fn search(term: &str) -> AHResult<String> {
        let https = HttpsConnector::new();
        let sender = Client::builder().build::<_, hyper::Body>(https);

        let request = Request::builder()
            .method(Method::POST)
            .uri("https://www.animeunity.tv/")
            .header("content-type", "application/json")
            .body(())?;

        let map = request.headers();

        for (key, value) in map.iter() {
            println!("{:?}: {:?}", key, value);
        }

        let body = json!({ "title": term }).to_string();

        let request = Request::builder()
            .method(Method::POST)
            .uri("https://www.animeunity.tv/livesearch")
            .header("content-type", "application/json")
            .body(Body::from(body))?;

        let mut response = sender.request(request).await?;
        let mut stuff = "".to_string();
        while let Some(chunk) = response.body_mut().data().await {
            let piece = String::from_utf8(chunk?.to_vec())?;
            stuff = format!("{stuff}{piece}");
        }

        //let output: Vec<Anime> = vec![];
        Ok(stuff)
    }

    pub async fn get_token(headless: bool) -> AHResult<String> {
        let mut server = crate::start_geckodriver().await?;

        let driver = crate::get_driver(headless).await?;

        driver
            .goto("https://www.animeunity.tv/anime/1469-naruto")
            .await?;

        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

        let requests = driver
            .execute("return window.performance.getEntries();", vec![])
            .await?;

        let requests = serde_json::to_string_pretty(requests.json())?;

        let requests = requests
            .split("token=")
            .last()
            .unwrap()
            .split("&expires")
            .next()
            .unwrap()
            .to_string();

        driver.quit().await?;
        server.kill().await?;
        Ok(requests)
    }
}
