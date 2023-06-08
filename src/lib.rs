pub mod core {
    extern crate headless_chrome;
    use headless_chrome::{Browser, LaunchOptions, Tab};
    extern crate scraper;
    use anyhow::{Error as AHError, Ok as AHOk, Result as AHResult};
    use core::fmt;
    use scraper::{ElementRef, Html, Selector};
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use serde_json::Value;
    use std::sync::Arc;

    pub fn create_browser() -> AHResult<Browser> {
        let launch_options = LaunchOptions::default_builder().build().unwrap();

        let browser = Browser::new(launch_options)?;
        Ok(browser)
    }

    pub fn create_tab(browser: &Browser) -> AHResult<Arc<Tab>> {
        let tab = browser.new_tab()?;
        Ok(tab)
    }

    pub trait EzHtmlLogic {
        fn ez_get_element(&self, selector: &str) -> AHResult<ElementRef, &str>;
    }

    impl EzHtmlLogic for Html {
        fn ez_get_element(&self, selector: &str) -> AHResult<ElementRef, &str> {
            let selector = match Selector::parse(selector) {
                Ok(t) => t,
                Err(t) => return Err("Invalid selector"),
            };
            let element = match self.select(&selector).next() {
                Some(e) => e,
                None => return Err("No element corresponding to the selector found"),
            };
            Ok(element)
        }
    }

    pub trait EzTabLogic {
        fn ez_get_source(&self) -> AHResult<Html, String>;
        fn ez_navigate(&self, url: &str) -> AHResult<(), String>;
        fn ez_wait_for_element(&self, selector: &str) -> AHResult<(), String>;
    }

    impl EzTabLogic for Tab {
        fn ez_get_source(&self) -> AHResult<Html, String> {
            let content = &self.get_content();
            let content = match content {
                Ok(e) => e,
                Err(e) => return Err(e.to_string()),
            };

            Ok(Html::parse_fragment(content))
        }
        fn ez_navigate(&self, url: &str) -> AHResult<(), String> {
            match &self.navigate_to(url) {
                Ok(e) => return Ok(()),
                Err(e) => return Err(e.to_string()),
            }
        }
        fn ez_wait_for_element(&self, selector: &str) -> AHResult<(), String> {
            match &self.wait_for_element(selector) {
                Ok(e) => return Ok(()),
                Err(e) => return Err(e.to_string()),
            }
        }
    }

    pub trait EzElementLogic {
        fn ez_get_attribute(&self, attribute: &str) -> AHResult<String, &str>;
        fn ez_get_innertext(&self) -> AHResult<String, &str>;
    }

    impl EzElementLogic for ElementRef<'_> {
        fn ez_get_attribute(&self, attribute: &str) -> AHResult<String, &str> {
            let attribute = match self.value().attr(attribute) {
                Some(e) => e,
                None => return Err("Error while getting attribute"),
            };
            Ok(String::from(attribute))
        }
        fn ez_get_innertext(&self) -> AHResult<String, &str> {
            let innertext = match self.text().next() {
                Some(e) => e,
                None => return Err("Error while getting innertext"),
            };
            Ok(String::from(innertext))
        }
    }

    #[derive(Clone, Deserialize, Serialize, fmt::Debug)]
    pub struct Anime<'a> {
        pub name: &'a str,
        pub link: &'a str,
        pub link_type: &'a str,
        pub total_episodes: usize,
        pub available_episodes: usize,
        pub image_path: &'a str,
    }

    impl Default for Anime<'static> {
        fn default() -> Anime<'static> {
            Anime {
                name: "",
                link: "",
                link_type: "",
                total_episodes: 0,
                available_episodes: 0,
                image_path: "",
            }
        }
    }

    pub trait AnimeStuff {
        fn from_json(json: &Value) -> Anime;
        fn to_json(&self) -> Value;
    }

    impl AnimeStuff for Anime<'_> {
        fn from_json(json: &Value) -> Anime {
            let object = json.as_object().unwrap();
            let key = object.keys().last().unwrap();

            let output = Anime {
                name: key,
                link: json[key]["link"].as_str().unwrap(),
                link_type: json[key]["link_type"].as_str().unwrap(),
                total_episodes: json[key]["total_episodes"].as_u64().unwrap() as usize,
                available_episodes: json[key]["available_episodes"].as_u64().unwrap() as usize,
                image_path: json[key]["image_path"].as_str().unwrap(),
            };

            output
        }
        fn to_json(&self) -> Value {
            let output = json!({
                * &self.name: {
                "link": self.link,
                "link_type": &self.link_type,
                "total_episodes": &self.total_episodes,
                "available_episodes": &self.available_episodes,
                "image_path": &self.image_path,
            }});

            return output;
        }
    }

    pub trait WebsiteScraper {
        fn get_episode_download_link(tab: Tab, anime_link: &str) -> AHResult<String, String>;
        fn get_title(tab: Tab, anime_link: &str) -> AHResult<String, String>;
    }
}

pub mod animeunity {
    use crate::core::{EzElementLogic, EzHtmlLogic, EzTabLogic};
    use anyhow::{Error as AHError, Ok as AHOk, Result as AHResult};
    use headless_chrome::Tab;

    pub const TYPE: (&str, AnimeUnity) = ("https://www.animeunity.tv/", AnimeUnity {});

    pub struct AnimeUnity {}

    impl crate::core::WebsiteScraper for AnimeUnity {
        fn get_episode_download_link(tab: Tab, anime_link: &str) -> AHResult<String, String> {
            tab.ez_navigate(anime_link)?;
            tab.ez_wait_for_element(".plyr__controls__item .plyr__control")?;
            let source = tab.ez_get_source()?;
            let element =
                source.ez_get_element("a[class=\"plyr__controls__item plyr__control\"]")?;
            let link = element.ez_get_attribute("href")?;
            Ok(link)
        }

        fn get_title(tab: Tab, anime_link: &str) -> AHResult<String, String> {
            tab.ez_navigate(anime_link)?;
            let source = tab.ez_get_source()?;
            let title = source.ez_get_element("h1.title")?.ez_get_innertext()?;
            Ok(title)
        }
    }
}
