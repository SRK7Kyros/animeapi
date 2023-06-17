use anyhow::{Error as AHError, Ok as AHOk, Result as AHResult};
use reqwest::{
    self,
    header::{self, HeaderMap, HeaderValue},
    Client,
};
use scraper::{Html, Selector};
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

pub async fn get_client() -> AHResult<Client> {
    let headers = HeaderMap::new();

    let client = Client::builder().default_headers(headers).build()?;
    Ok(client)
}

pub async fn get_csrf_token(html: String) -> AHResult<String> {
    let html = Html::parse_document(html.as_str());
    let selector = Selector::parse("meta[name='csrf-token']").unwrap();

    let csrf_token = html
        .select(&selector)
        .next()
        .unwrap()
        .value()
        .attr("content")
        .unwrap();
    Ok(csrf_token.to_string())
}

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

#[derive(Clone, Deserialize, Serialize, fmt::Debug, Default)]
pub struct Anime {
    pub name: String,
    pub link: String,
    pub link_type: String,
    pub total_episodes: usize,
    pub available_episodes: usize,
    pub image_path: String,
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
    use crate::{get_client, get_csrf_token, Anime, AnimeStuff};
    use anyhow::{Error as AHError, Ok, Result as AHResult};
    use reqwest::header::{self, HeaderMap, COOKIE};
    use scraper::{Html, Selector};
    use serde_json::{self, Value};
    use serde_json::{from_str, json};
    use std::{fmt::format, process::Output, thread, vec};
    use thirtyfour::prelude::*;
    use tokio::net::TcpStream;

    pub async fn search(term: &str) -> AHResult<Value> {
        let client = get_client().await?;

        let html_res = client.get("https://www.animeunity.it").send().await?;
        let html_res_headers = html_res.headers().clone();

        let mut search_req_headers = HeaderMap::new();
        let mut html_res_cookies = html_res_headers.get_all("set-cookie").iter();
        let xsrf_token = html_res_cookies.next().unwrap().to_str()?;
        let animeunity_session = html_res_cookies.next().unwrap().to_str()?;
        let cookie = format!("{xsrf_token};{animeunity_session}");
        search_req_headers.insert(COOKIE, cookie.parse().unwrap());

        let body = html_res.text().await?;
        let csrf_token = get_csrf_token(body).await?;

        search_req_headers.insert("X-Requested-With", "XMLHttpRequest".parse().unwrap());
        search_req_headers.insert("X-CSRF-TOKEN", csrf_token.parse().unwrap());
        search_req_headers.insert("Content-Type", "application/json".parse().unwrap());

        let search_req_body = json!({ "title": term }).to_string();
        let search_req = client
            .post("https://www.animeunity.it/livesearch")
            .body(search_req_body)
            .headers(search_req_headers);

        let search_res = search_req.send().await?;

        let search_res_json = search_res.json().await?;

        let output: Vec<Anime> = vec![];
        Ok(search_res_json)
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
