use anyhow::{Error as AHError, Ok as AHOk, Result as AHResult};
use hyper::{body::HttpBody, http::request::Builder, Body, Request, Response};
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

pub async fn merge(a: &mut Value, b: &Value) {
    match (a, b) {
        (&mut Value::Object(ref mut a), &Value::Object(ref b)) => {
            for (k, v) in b {
                merge(a.entry(k.clone()).or_insert(Value::Null), v);
            }
        }
        (a, b) => {
            *a = b.clone();
        }
    }
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

pub async fn get_request_headers(
    request: &mut Request<Body>,
    name: Option<&str>,
) -> AHResult<Value> {
    let mut headers = json!({});
    let headers = headers.as_object_mut().unwrap();
    for (key, value) in request.headers().iter() {
        let value = Value::String(value.to_str().unwrap().to_string());
        headers.insert(key.to_string(), value);
    }
    let output = match name {
        Some(e) => {
            let output = json!({
                e: Value::Object(headers.to_owned())
            });
            output
        }
        None => {
            let output: Value = Value::Object(headers.to_owned());
            output
        }
    };
    Ok(output)
}

pub async fn get_response_headers(
    response: &mut Response<Body>,
    name: Option<&str>,
) -> AHResult<Value> {
    let mut headers = json!({});
    let headers = headers.as_object_mut().unwrap();
    for (key, value) in response.headers().iter() {
        let value = Value::String(value.to_str().unwrap().to_string());
        headers.insert(key.to_string(), value);
    }
    let output = match name {
        Some(e) => {
            let output = json!({
                e: Value::Object(headers.to_owned())
            });
            output
        }
        None => {
            let output: Value = Value::Object(headers.to_owned());
            output
        }
    };
    Ok(output)
}

pub async fn get_response_body(response: &mut Response<Body>) -> AHResult<String> {
    let mut stuff = "".to_string();

    while let Some(chunk) = response.body_mut().data().await {
        let bytes = &chunk?.to_vec();
        stuff.push_str(std::str::from_utf8(&bytes)?);
    }
    Ok(stuff)
}

pub async fn get_request_with_headers() -> AHResult<Builder> {
    let request = Request::builder()
        .header("Content-Type", "application/json;charset=utf-8")
        .header("Accept", "*/*")
        // .header("Accept-Encoding", "gzip, deflate, br")
        .header("Connection", "keep-alive")
        .header("User-Agent", "PostmanRuntime/7.32.2");

    let output = request;
    Ok(output)
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
    use crate::{
        get_csrf_token, get_request_headers, get_request_with_headers, get_response_body,
        get_response_headers, merge, Anime,
    };
    use anyhow::{Ok, Result as AHResult};
    use hyper::{body::HttpBody, Body, Client, Method, Request};
    use hyper_tls::HttpsConnector;
    use scraper::{Html, Selector};
    use serde_json::json;
    use serde_json::{self, Value};
    use std::{fmt::format, process::Output, thread, vec};
    use thirtyfour::prelude::*;
    use tokio::net::TcpStream;

    pub async fn search(term: &str) -> AHResult<(std::string::String, Value)> {
        let https = HttpsConnector::new();
        let sender = Client::builder().build::<_, hyper::Body>(https);

        let mut req = get_request_with_headers()
            .await?
            .method(Method::GET)
            .uri("https://www.animeunity.tv/")
            .body(Body::empty())?;

        let mut richiesta1_headers = get_request_headers(&mut req, Some("richiesta 1")).await?;

        let mut res = sender.request(req).await?;

        let body = get_response_body(&mut res).await?;
        let csrf_token = get_csrf_token(body).await?;

        let risposta1_headers = get_response_headers(&mut res, Some("risposta 1")).await?;
        let cookie = &risposta1_headers["set-cookie"].to_string();

        let body = json!({ "title": term }).to_string();

        let mut req = get_request_with_headers()
            .await?
            .method(Method::POST)
            .uri("https://www.animeunity.tv/livesearch")
            .header("X-Requested-With", "XMLHttpRequest")
            .header("X-CSRF-TOKEN", csrf_token)
            .header("Cookie", cookie)
            .header("Host", "www.animeunity.tv")
            .body(Body::from(body))?;

        let richiesta2_headers = get_request_headers(&mut req, Some("richiesta 2")).await?;

        let mut res = sender.request(req).await?;
        let body = get_response_body(&mut res).await?;

        let risposta2_headers = get_response_headers(&mut res, Some("risposta 2")).await?;

        merge(&mut richiesta1_headers, &risposta1_headers).await;
        merge(&mut richiesta1_headers, &richiesta2_headers).await;
        merge(&mut richiesta1_headers, &risposta2_headers).await;

        let output: Vec<Anime> = vec![];
        Ok((body, richiesta1_headers))
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
