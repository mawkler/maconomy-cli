use crate::config::Configuration;
use ::futures::StreamExt;
use anyhow::{anyhow, bail, Context, Result};
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::network::Cookie;
use chromiumoxide::page::Page;
use log::info;
use serde::Deserialize;
use std::fmt::Display;
use std::io::BufReader;
use std::{fs::File, io::Write, time::Duration};

const COOKIE_NAME_PREFIX: &str = "Maconomy-";
const COOKIE_FILE_NAME: &str = "maconomy_cookie";
const TIMEOUT: Duration = Duration::from_secs(300);
const POLL_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct AuthCookie {
    pub name: String,
    pub value: String,
}

impl From<Cookie> for AuthCookie {
    fn from(cookie: Cookie) -> Self {
        AuthCookie {
            name: cookie.name.to_string(),
            value: cookie.value.to_string(),
        }
    }
}

impl Display for AuthCookie {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}={}", self.name, self.value)
    }
}

pub(crate) struct AuthService {
    config: Configuration,
    auth_cookie: Option<AuthCookie>,
    login_url: String,
}

impl AuthService {
    pub(crate) fn new(login_url: String, config: Configuration) -> Self {
        Self {
            config,
            auth_cookie: None,
            login_url,
        }
    }

    pub(crate) async fn authenticate(&self) -> Result<AuthCookie> {
        if let Some(cookie) = &self.auth_cookie {
            info!("Found service cookie in memory");
            return Ok(cookie.clone());
        }

        info!("Cookie not found in memory, attempting to read in from file");
        if let Some(cookie) = read_cookie_from_file()? {
            info!("Found cookie in file");
            return Ok(cookie.clone());
        }

        info!("Cookie file not found, opening browser");
        self.reauthenticate().await
    }

    pub(crate) async fn reauthenticate(&self) -> Result<AuthCookie> {
        let cookie = self.open_browser_and_authenticate().await?;

        write_cookie_to_file(&cookie)?;

        Ok(cookie.into())
    }

    async fn open_browser_and_authenticate(&self) -> Result<Cookie> {
        let config = BrowserConfig::builder()
            .with_head()
            .build()
            .map_err(|err| anyhow!("Failed to create browser config: {err}"))?;
        let (mut browser, mut handler) = Browser::launch(config)
            .await
            .context("Failed to launch web browser")?;

        let handle = tokio::task::spawn(async move {
            while let Some(h) = handler.next().await {
                if h.is_err() {
                    break;
                }
            }
        });

        let page = browser
            .new_page(&self.login_url)
            .await
            .context("Failed to create new web page")?;

        let auth_cookie = wait_for_auth_cookie(&page).await?;

        browser.close().await?;
        let _ = handle.await;

        Ok(auth_cookie)
    }
}

fn write_cookie_to_file(cookie: &Cookie) -> Result<()> {
    // TODO: use `Serialize` instead
    let cookie = serde_json::json!({
        "name": cookie.name,
        "value": cookie.value,
    })
    .to_string();

    let mut file = File::create(COOKIE_FILE_NAME).context("Failed to create cookie file")?;
    file.write_all(cookie.as_bytes())
        .context("Failed to write cookie to file")?;

    Ok(())
}

// TODO: change Result to a cleaner type that is NotFound or Other
fn read_cookie_from_file() -> Result<Option<AuthCookie>> {
    let file = match File::open(COOKIE_FILE_NAME) {
        Ok(file) => file,
        Err(_) => return Ok(None),
    };

    let reader = BufReader::new(file);
    let cookie: AuthCookie =
        serde_json::from_reader(reader).context("Failed to deserialize cookie from file")?;

    Ok(Some(cookie))
}

async fn get_maconomy_cookie(page: &Page) -> Option<Cookie> {
    page.get_cookies()
        .await
        .ok()?
        .into_iter()
        // Could there be more than one maconomy cookie?
        .find(|c| c.name.starts_with(COOKIE_NAME_PREFIX))
}

async fn wait_for_auth_cookie(page: &Page) -> Result<Cookie> {
    let start_time = std::time::Instant::now();

    loop {
        if start_time.elapsed() > TIMEOUT {
            bail!("Timed out waiting for user to sign in");
        }

        let cookies = get_maconomy_cookie(page).await;
        if let Some(cookie) = cookies {
            return Ok(cookie);
        }

        tokio::time::sleep(POLL_INTERVAL).await;
    }
}
