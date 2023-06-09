pub mod core {
    extern crate headless_chrome;
    use headless_chrome::Element;
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
        let mut launch_options = LaunchOptions::default_builder();
        launch_options.headless(false);

        let launch_options = launch_options.build().unwrap();

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
        fn ez_wait_for_element(&self, selector: &str) -> AHResult<Element, String>;
        fn ez_type_str(&self, string_to_type: &str) -> AHResult<&Self, String>;
    }

    impl EzTabLogic for Tab {
        fn ez_get_source(&self) -> AHResult<Html, String> {
            let content = self.get_content();
            let content = match content {
                Ok(e) => e,
                Err(e) => return Err(e.to_string()),
            };

            Ok(Html::parse_fragment(content.as_str()))
        }
        fn ez_navigate(&self, url: &str) -> AHResult<(), String> {
            match self.navigate_to(url) {
                Ok(e) => return Ok(()),
                Err(e) => return Err(e.to_string()),
            }
        }
        fn ez_wait_for_element(&self, selector: &str) -> AHResult<Element, String> {
            let element = match self.wait_for_element(selector) {
                Ok(e) => e,
                Err(e) => return Err(e.to_string()),
            };
            Ok(element)
        }
        fn ez_type_str(&self, string_to_type: &str) -> AHResult<&Self, String> {
            match self.type_str(string_to_type) {
                Ok(e) => return Ok(e),
                Err(e) => return Err(e.to_string()),
            }
        }
    }

    pub trait EzElementRefLogic {
        fn ez_get_attribute(&self, attribute: &str) -> AHResult<String, &str>;
        fn ez_get_innertext(&self) -> AHResult<String, &str>;
    }

    impl EzElementRefLogic for ElementRef<'_> {
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

    pub trait EzElementLogic {
        fn ez_click(&self) -> AHResult<(), String>;
    }

    impl EzElementLogic for Element<'_> {
        fn ez_click(&self) -> AHResult<(), String> {
            match self.click() {
                Ok(e) => return Ok(()),
                Err(e) => return Err(e.to_string()),
            }
        }
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

    pub trait WebsiteScraper {
        fn query(tab: Arc<Tab>, input: String) -> Result<Vec<Anime>, String>;
        fn get_episode_download_link(tab: Arc<Tab>) -> AHResult<String, String>;
        fn get_title(tab: Arc<Tab>) -> AHResult<String, String>;
    }
}

pub mod animeunity {
    use crate::core::{Anime, EzElementLogic, EzElementRefLogic, EzHtmlLogic, EzTabLogic};
    use anyhow::{Error as AHError, Ok as AHOk, Result as AHResult};
    use headless_chrome::Tab;
    use std::sync::Arc;

    pub const TYPE: (&str, AnimeUnity) = ("https://www.animeunity.tv/", AnimeUnity {});

    pub struct AnimeUnity {}

    impl crate::core::WebsiteScraper for AnimeUnity {
        fn get_episode_download_link(tab: Arc<Tab>) -> AHResult<String, String> {
            let source = tab.ez_get_source()?;
            let element =
                source.ez_get_element("a[class=\"plyr__controls__item plyr__control\"]")?;
            let link = element.ez_get_attribute("href")?;
            Ok(link)
        }

        fn get_title(tab: Arc<Tab>) -> AHResult<String, String> {
            let source = tab.ez_get_source()?;
            let title = source.ez_get_element("h1.title")?.ez_get_innertext()?;
            Ok(title)
        }

        fn query(tab: Arc<Tab>, input: String) -> Result<Vec<Anime>, String> {
            let searchbutton = tab.ez_wait_for_element("fas fa-search text-white")?;
            searchbutton.ez_click()?;

            let searchbar = tab.ez_wait_for_element("search-bar")?;
            searchbar.ez_click()?;
            tab.ez_type_str(input.as_str())?;

            let output: Vec<Anime> = vec![];
            Ok(output)
        }
    }
}
