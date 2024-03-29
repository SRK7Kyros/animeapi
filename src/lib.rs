use anyhow::{Error as AHError, Result as AHResult};
use reqwest::{self, header::HeaderMap, Client};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::future::Future;
use std::time::Instant;
use thirtyfour::prelude::*;
use tokio::process::Child;

pub async fn time<T: Future>(
    fn_name: &str,
    future: T,
) -> AHResult<(String, <T as Future>::Output)> {
    let now = Instant::now();
    let result = future.await;
    let elapsed = now.elapsed();
    Ok((format!("{} lasted {:.2?}", fn_name, elapsed), result))
}

pub async fn merge(a: &mut Value, b: &mut Value) -> AHResult<()> {
    let a = a.as_object_mut().ok_or(AHError::msg(
        "Could not convert the first Value to an object",
    ))?;
    let b = b.as_object_mut().ok_or(AHError::msg(
        "Could not convert the second Value to an object",
    ))?;
    a.append(b);

    Ok(())
}

pub async fn get_client() -> AHResult<Client> {
    let headers = HeaderMap::new();

    let client = Client::builder()
        .default_headers(headers)
        .cookie_store(true)
        .build()?;
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

#[derive(Clone, Deserialize, Serialize, Default)]
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
    use crate::{get_client, get_csrf_token};
    use anyhow::{Error as AHError, Ok, Result as AHResult};
    use reqwest::cookie::{CookieStore, Jar};
    use reqwest::header::HeaderMap;
    use reqwest::{Client, Url};
    use serde::{Deserialize, Serialize};
    use serde_aux::field_attributes::deserialize_number_from_string;
    use serde_json::{self, Value};
    use serde_json::{json, to_string_pretty};

    use std::{fmt::Debug, vec};

    #[derive(Clone, Serialize, Deserialize, Debug)]
    pub enum EntryType {
        TV,
        Movie,
        OVA,
    }

    #[derive(Clone, Serialize, Deserialize, Debug)]
    pub struct SearchEntry {
        #[serde(rename(deserialize = "title_eng"))]
        title: String,
        episodes_count: usize,
        #[serde(deserialize_with = "deserialize_number_from_string")]
        date: usize,
        #[serde(rename = "type")]
        entry_type: EntryType,
        #[serde(rename(deserialize = "imageurl"))]
        image_url: String,
        slug: String,
        id: usize,
    }

    pub async fn get_animeunity_client() -> AHResult<Client> {
        let mut headers = HeaderMap::new();
        let jar = Jar::default();

        let html_res = reqwest::get("https://www.animeunity.it").await?;

        let mut set_cookie_headers = html_res.headers().get_all("set-cookie").iter();
        jar.set_cookies(
            &mut set_cookie_headers,
            &"https://www.animeunity.to".parse::<Url>()?,
        );

        let body = html_res.text().await?;
        let csrf_token = get_csrf_token(body).await?;

        headers.insert("X-Requested-With", "XMLHttpRequest".parse().unwrap());
        headers.insert("X-CSRF-TOKEN", csrf_token.parse().unwrap());
        headers.insert("Content-Type", "application/json".parse().unwrap());
        let client = Client::builder()
            .default_headers(headers)
            .cookie_provider(jar.into())
            .build()?;
        Ok(client)
    }

    pub async fn search(term: &str, client: &Client) -> AHResult<Value> {
        let search_req_body = json!({
            "title": term,
            "type":false,
            "year":false,
            "order":false,
            "status":false,
            "genres":false,
            "offset":0,
            "dubbed":false,
            "season":false});

        let search_req = client
            .post("https://www.animeunity.to/archivio/get-animes")
            .json(&search_req_body);

        let search_res = search_req.send().await?;

        let search_res_json = search_res.json::<Value>().await?;
        let records = search_res_json
            .get("records")
            .ok_or(AHError::msg("No records obtained"))?
            .to_owned();

        Ok(records)
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
