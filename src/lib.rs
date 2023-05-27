pub mod core {
    extern crate headless_chrome; use headless_chrome::{Browser, LaunchOptions, Tab};
    extern crate scraper; use scraper::{Html, Selector, ElementRef};
    use anyhow::Result;
    use serde::{Serialize, Deserialize};
    use std::{sync::Arc};

    pub fn create_browser() -> Result<Browser> {
        let launch_options = LaunchOptions::default_builder()
            .build()
            .unwrap();
    
        let browser = Browser::new(launch_options)?;
        Ok(browser)
    }
    
    pub fn create_tab(browser: &Browser) -> Result<Arc<Tab>> {
        let tab = browser.new_tab()?;
        Ok(tab)
    }
    
    
    pub trait EzHtmlLogic {
        fn ez_get_element(&self, selector: &str) -> Result<ElementRef>;
    }
    
    impl EzHtmlLogic for Html {
        fn ez_get_element(&self, selector: &str) -> Result<ElementRef> {
            let selector = Selector::parse(selector).unwrap();
            let element = self.select(&selector).next().unwrap();
            Ok(element)
        }
    }
    
    
    pub trait EzTabLogic {
        fn ez_get_source(&self) -> Result<Html>;
    }
    
    impl EzTabLogic for Tab {
        fn ez_get_source(&self) -> Result<Html> {
            Ok(Html::parse_fragment(&self.get_content()?))
        }
    }
    
    
    pub trait EzElementLogic {
        fn ez_get_attribute(&self, attribute: &str) -> Result<String>;
        fn ez_get_innertext(&self) -> Result<String>;
    }
    
    impl EzElementLogic for ElementRef<'_> {
        fn ez_get_attribute(&self, attribute: &str) -> Result<String> {
            let attribute = self.value().attr(attribute).unwrap();
            Ok(String::from(attribute))
        }
        
        fn ez_get_innertext(&self) -> Result<String> {
            let innertext = self.text().next().unwrap();
            Ok(String::from(innertext))
        }
    
    }
    
    #[derive(Clone, Deserialize, Serialize)]
    pub struct Anime<'a> (&'a str, AnimeData<'a>);
        
    #[derive(Clone, Deserialize, Serialize)]
    pub struct AnimeData<'a> {
        pub link: &'a str,
        pub link_type: &'a str,
        pub total_episodes: usize,
        pub available_episodes: usize,
        pub image_path: &'a str,
    }
    
    impl Default for Anime<'_> {
        fn default() -> Anime<'static> {
            Anime (
                "", 
                AnimeData {
                    link: "",
                    link_type: "",
                    total_episodes: 0,
                    available_episodes: 0,
                    image_path: "",
                }
            )
        }
    }
    
    pub trait WebsiteScraper {
        fn get_episode_download_link(tab: Tab, anime_link: &str) -> Result<String>;
        fn get_title(tab: Tab, anime_link: &str) -> Result<String>;
    }
}

pub mod animeunity {
    use crate::core::{EzTabLogic, EzHtmlLogic, EzElementLogic};
    use anyhow::Result;
    use headless_chrome::Tab;

    pub const TYPE: (&str, AnimeUnity) = ("https://www.animeunity.tv/", AnimeUnity {});
    
    pub struct AnimeUnity {}

    impl crate::core::WebsiteScraper for AnimeUnity {
        fn get_episode_download_link(tab: Tab, anime_link: &str) -> Result<String> {
            tab.navigate_to(anime_link)?;
            tab.wait_for_element(".plyr__controls__item .plyr__control")?;
            let source = tab.ez_get_source()?;
            let element = source.ez_get_element("a[class=\"plyr__controls__item plyr__control\"]")?;
            let link = element.ez_get_attribute("href")?;
            Ok(link)
        }
    
        fn get_title(tab: Tab, anime_link: &str) -> Result<String> {
            tab.navigate_to(anime_link)?;
            let source = tab.ez_get_source()?;
            let title = source.ez_get_element("h1.title")?.ez_get_innertext()?;
            Ok(title)
        }
    }
}
