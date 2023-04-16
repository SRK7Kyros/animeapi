extern crate headless_chrome; use headless_chrome::{Browser, LaunchOptions, Tab};
extern crate scraper; use scraper::{Html, Selector, ElementRef};
use std::sync::Arc;

pub fn create_browser() -> Browser {
    let launch_options = LaunchOptions::default_builder()
        .build()
        .unwrap();

    let browser = match Browser::new(launch_options) {
        Ok(browser) => browser,
        Err(e) => {
            eprintln!("Failed to create the browser: {}", e);
            panic!();        
        }
    };
    return browser;
}

pub fn create_tab(browser: &Browser) -> Arc<Tab> {
    let tab = match browser.new_tab() {
        Ok(tab) => tab,
        Err(e) => {
            eprintln!("Failed to create the browser: {}", e);
            panic!();
        }
    };
    return tab;
}

trait HtmlStuff {
    fn get_element(&self, selector: &str) -> ElementRef;
}

impl HtmlStuff for Html {
    fn get_element(&self, selector: &str) -> ElementRef {
        let selector = match Selector::parse(selector) {
            Ok(selector) => selector,
            Err(e) => {
                eprintln!("Ran into an error while instantiating the selector: {}", e);
                panic!();
            }
        };
    
        self.select(&selector).next().expect("No element found")
    }
}

trait TabStuff {
    fn ez_navigate(&self, url: &str);
    fn ez_wait_4_element(&self, selector: &str);
    fn get_source(&self) -> Html;
}

impl TabStuff for Tab {
    fn ez_navigate(&self, url: &str) {
        match &self.navigate_to(url) {
            Ok(tab) => tab,
            Err(e) => {
                panic!("Navigation failed: {}", e);
            }
        };
    } 

    fn ez_wait_4_element(&self, selector: &str) {
        match &self.wait_for_element(selector) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Ran into an error while waiting the element: {}", e);
                panic!();
            }
        }
    }

    fn get_source(&self) -> Html{
        let tab = &self;
        let content = match tab.get_content() {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Ran into an error while fetching the page source: {}", e);
                panic!();
            }
        };
    
        return Html::parse_fragment(&content);
    }
}

trait ElementStuff {
    fn get_attribute(&self, attribute: &str) -> &str;
}

impl ElementStuff for ElementRef<'_> {
    fn get_attribute(&self, attribute: &str) -> &str {
        self.value().attr(attribute).expect("No corresponding attribute found")
    }
}

pub mod animeunity {
    use std::sync::Arc;

    use headless_chrome::{Tab};
    use crate::{TabStuff, HtmlStuff, ElementStuff};
    
    pub fn get_episode_download_link(tab: Arc<Tab>, anime_link: &str) -> String {
        tab.ez_navigate(anime_link);

        tab.ez_wait_4_element(".plyr__controls__item .plyr__control");
        
        let source = tab.get_source();

        let element = source.get_element("a[class=\"plyr__controls__item plyr__control\"]");

        let link: String = String::from(element.get_attribute("href"));

        return link;
        }
}
