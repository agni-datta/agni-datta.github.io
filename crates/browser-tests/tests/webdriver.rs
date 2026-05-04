use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use std::env;
use std::thread;
use std::time::{Duration, Instant};

const ELEMENT_KEY: &str = "element-6066-11e4-a52e-4f735466cecf";

struct Browser {
    endpoint: String,
    session_id: String,
}

impl Browser {
    fn connect() -> Result<Self> {
        let endpoint =
            env::var("WEBDRIVER_URL").unwrap_or_else(|_| "http://localhost:4444".to_owned());
        let browser = env::var("WEBDRIVER_BROWSER").unwrap_or_else(|_| "safari".to_owned());
        let payload = json!({
            "capabilities": {
                "alwaysMatch": {
                    "browserName": browser,
                    "pageLoadStrategy": "normal"
                }
            }
        });
        let response = request_json("POST", &format!("{endpoint}/session"), Some(payload))?;
        let session_id = response
            .pointer("/value/sessionId")
            .or_else(|| response.get("sessionId"))
            .and_then(Value::as_str)
            .context("WebDriver did not return a session identifier")?
            .to_owned();
        Ok(Self {
            endpoint,
            session_id,
        })
    }

    fn command(&self, method: &str, command: &str, body: Option<Value>) -> Result<Value> {
        request_json(
            method,
            &format!("{}/session/{}/{}", self.endpoint, self.session_id, command),
            body,
        )
    }

    fn navigate(&self, url: &str) -> Result<()> {
        self.command("POST", "url", Some(json!({ "url": url })))?;
        Ok(())
    }

    fn refresh(&self) -> Result<()> {
        self.command("POST", "refresh", Some(json!({})))?;
        Ok(())
    }

    fn back(&self) -> Result<()> {
        self.command("POST", "back", Some(json!({})))?;
        Ok(())
    }

    fn find(&self, selector: &str) -> Result<String> {
        let response = self.command(
            "POST",
            "element",
            Some(json!({ "using": "css selector", "value": selector })),
        )?;
        response
            .pointer(&format!("/value/{ELEMENT_KEY}"))
            .and_then(Value::as_str)
            .map(str::to_owned)
            .with_context(|| format!("could not find `{selector}`"))
    }

    fn click(&self, selector: &str) -> Result<()> {
        let element = self.find(selector)?;
        self.command("POST", &format!("element/{element}/click"), Some(json!({})))?;
        Ok(())
    }

    fn attribute(&self, selector: &str, attribute: &str) -> Result<Option<String>> {
        let element = self.find(selector)?;
        let response = self.command(
            "GET",
            &format!("element/{element}/attribute/{attribute}"),
            None,
        )?;
        Ok(response
            .pointer("/value")
            .and_then(Value::as_str)
            .map(str::to_owned))
    }

    fn wait_attribute(&self, selector: &str, attribute: &str, expected: &str) -> Result<()> {
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if self.attribute(selector, attribute)?.as_deref() == Some(expected) {
                return Ok(());
            }
            thread::sleep(Duration::from_millis(50));
        }
        bail!("`{selector}` did not reach {attribute}={expected}")
    }

    fn cookie(&self, name: &str) -> Result<Value> {
        self.command("GET", &format!("cookie/{name}"), None)
    }

    fn viewport(&self, width: u16, height: u16) -> Result<()> {
        self.command(
            "POST",
            "window/rect",
            Some(json!({ "width": width, "height": height })),
        )?;
        Ok(())
    }

    fn send_escape(&self) -> Result<()> {
        let body = self.find("body")?;
        self.command(
            "POST",
            &format!("element/{body}/value"),
            Some(json!({ "text": "\u{e00c}", "value": ["\u{e00c}"] })),
        )?;
        Ok(())
    }
}

impl Drop for Browser {
    fn drop(&mut self) {
        let _ = request_json(
            "DELETE",
            &format!("{}/session/{}", self.endpoint, self.session_id),
            None,
        );
    }
}

fn request_json(method: &str, url: &str, body: Option<Value>) -> Result<Value> {
    let response = match method {
        "GET" => ureq::get(url).call(),
        "POST" => ureq::post(url).send_json(body.unwrap_or_else(|| json!({}))),
        "DELETE" => ureq::delete(url).call(),
        _ => bail!("unsupported WebDriver method"),
    }
    .with_context(|| format!("WebDriver request failed at {url}"))?;
    let mut response = response;
    let payload: Value = response.body_mut().read_json()?;
    if payload.pointer("/value/error").is_some() {
        bail!("WebDriver error response: {payload}");
    }
    Ok(payload)
}

#[test]
#[ignore = "requires cargo site serve and a local WebDriver"]
fn theme_cookie_survives_spa_refresh_direct_load_and_history() -> Result<()> {
    let browser = Browser::connect()?;
    browser.navigate("http://localhost:8000/")?;
    browser.wait_attribute("html", "data-theme", "dark")?;
    browser.click("[data-theme-toggle]")?;
    browser.wait_attribute("html", "data-theme", "light")?;
    let cookie = browser.cookie("theme")?;
    assert_eq!(
        cookie.pointer("/value/value").and_then(Value::as_str),
        Some("light")
    );

    browser.click(".site-nav [data-route='publications']")?;
    browser.wait_attribute("main#content", "data-route", "publications")?;
    browser.wait_attribute("html", "data-theme", "light")?;
    browser.refresh()?;
    browser.wait_attribute("html", "data-theme", "light")?;

    browser.navigate("http://localhost:8000/notes/")?;
    browser.wait_attribute("main#content", "data-route", "notes")?;
    browser.wait_attribute("html", "data-theme", "light")?;
    browser.back()?;
    browser.wait_attribute("main#content", "data-route", "publications")?;
    Ok(())
}

#[test]
#[ignore = "requires cargo site serve and a local WebDriver"]
fn mobile_drawer_closes_with_escape_and_backdrop() -> Result<()> {
    let browser = Browser::connect()?;
    browser.viewport(390, 844)?;
    browser.navigate("http://localhost:8000/")?;
    browser.wait_attribute("html", "data-theme", "dark")?;

    browser.click("[data-drawer-toggle]")?;
    browser.wait_attribute("html", "data-navigation", "open")?;
    browser.send_escape()?;
    browser.wait_attribute("html", "data-navigation", "closed")?;
    assert_eq!(
        browser
            .attribute("[data-drawer-toggle]", "aria-expanded")?
            .as_deref(),
        Some("false")
    );

    browser.click("[data-drawer-toggle]")?;
    browser.wait_attribute("html", "data-navigation", "open")?;
    browser.click("[data-drawer-backdrop]")?;
    browser.wait_attribute("html", "data-navigation", "closed")?;
    Ok(())
}
