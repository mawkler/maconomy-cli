use anyhow::{bail, Context, Result};
use chromiumoxide::cdp::browser_protocol::network::{ClearBrowserCookiesParams, Cookie};
use chromiumoxide::page::Page;
use futures::StreamExt;
use log::{debug, error};
use serde::Deserialize;
use std::borrow::Cow;
use std::io::{self, BufReader};
use std::{fmt::Display, fs};
use std::{fs::File, io::Write, time::Duration};

const COOKIE_NAME_PREFIX: &str = "Maconomy-";
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
    auth_cookie: Option<AuthCookie>,
    login_url: String,
    cookie_path: String,
}

impl AuthService {
    pub(crate) fn new(login_url: String, cookie_path: String) -> Self {
        Self {
            auth_cookie: None,
            login_url,
            cookie_path,
        }
    }

    pub(crate) async fn authenticate(&self) -> Result<AuthCookie> {
        if let Some(cookie) = &self.auth_cookie {
            debug!("Found auth cookie in memory");
            return Ok(cookie.clone());
        }

        debug!("Cookie not found in memory, attempting to read in from file");
        if let Some(cookie) = self.read_cookie_from_file()? {
            debug!("Found auth cookie in file");
            return Ok(cookie.clone());
        }

        debug!("Cookie file not found, attempting to reauthenticate");
        eprintln!("User not logged in. Opening web browser...");
        self.reauthenticate()
            .await
            .context("Failed to reauthenticate")
    }

    pub(crate) async fn reauthenticate(&self) -> Result<AuthCookie> {
        let cookie = self
            .open_browser_and_authenticate()
            .await
            .context("Failed to authenticate user through web browser")?;

        self.write_cookie_to_file(&cookie)?;

        Ok(cookie.into())
    }

    pub(crate) async fn logout(&self) -> Result<()> {
        if let Err(err) = fs::remove_file(&*self.get_cookie_path()?) {
            if err.kind() != io::ErrorKind::NotFound {
                bail!("Failed to remove auth cookie: {err}");
            }
        };

        self.clear_browser_cookies()
            .await
            .context("Failed to clear browser cookies")
    }

    async fn open_browser_and_authenticate(&self) -> Result<Cookie> {
        let (mut browser, mut handler) = Self::launch_browser(true).await?;
        let handle = tokio::task::spawn(async move {
            while let Some(result) = handler.next().await {
                if let Err(err) = result {
                    error!("{err}");
                }
            }
        });

        let page = browser
            .new_page(&self.login_url)
            .await
            .context("Failed to create new web page")?;

        let auth_cookie = wait_for_auth_cookie(&page).await?;

        browser.close().await.context("Failed to close browser")?;
        let _ = handle.await;

        Ok(auth_cookie)
    }

    async fn launch_browser(
        with_head: bool,
    ) -> Result<(chromiumoxide::Browser, chromiumoxide::Handler)> {
        let mut builder = chromiumoxide::BrowserConfig::builder();
        if with_head {
            debug!("Configuring browser with head");
            builder = builder.with_head();
        } else {
            debug!("Configuring browser with no head");
        }

        let browser_config = match builder.build() {
            Ok(config) => config,
            Err(err) => {
                error!("Failed to create browser config: {err}");
                panic!(
                    "Failed to create browser config. Please make sure that you have either \
                    Chromium or Google Chrome installed"
                );
            }
        };

        chromiumoxide::Browser::launch(browser_config)
            .await
            .context("Failed to launch browser")
    }

    async fn clear_browser_cookies(&self) -> Result<()> {
        let (browser, mut handler) = Self::launch_browser(false).await?;

        tokio::spawn(async move { while (handler.next().await).is_some() {} });
        let _ = browser
            .new_page("about:blank")
            .await
            .context("Failed to create new page")?
            .execute(ClearBrowserCookiesParams::default())
            .await
            .context("Failed to execute clearing of browser cookies")?;

        Ok(())
    }

    fn get_cookie_path(&self) -> Result<Cow<str>> {
        shellexpand::full(&self.cookie_path).context("Failed to expand cookie path")
    }

    fn write_cookie_to_file(&self, cookie: &Cookie) -> Result<()> {
        let cookie = serde_json::json!({
            "name": cookie.name,
            "value": cookie.value,
        })
        .to_string();

        let mut file =
            File::create(&*self.get_cookie_path()?).context("Failed to create cookie file")?;
        file.write_all(cookie.as_bytes())
            .context("Failed to write cookie to file")?;

        Ok(())
    }

    fn read_cookie_from_file(&self) -> Result<Option<AuthCookie>> {
        let file = match File::open(&*self.get_cookie_path()?) {
            Ok(file) => file,
            Err(_) => return Ok(None),
        };

        let reader = BufReader::new(file);
        let cookie: AuthCookie =
            serde_json::from_reader(reader).context("Failed to deserialize cookie from file")?;

        Ok(Some(cookie))
    }
}

async fn get_maconomy_cookie(page: &Page) -> Result<Option<Cookie>> {
    let cookies = page
        .get_cookies()
        .await
        .context("failed to get cookies")?
        .into_iter()
        // Could there be more than one maconomy cookie?
        // TODO: fetch the name of the cookie from the Maconomy-Cookie header, and use that to make
        // sure that we get the right cookie
        .find(|c| c.name.starts_with(COOKIE_NAME_PREFIX));
    Ok(cookies)
}

async fn wait_for_auth_cookie(page: &Page) -> Result<Cookie> {
    let start_time = std::time::Instant::now();

    loop {
        if start_time.elapsed() > TIMEOUT {
            bail!("Timed out waiting for user to sign in");
        }

        if let Some(cookie) = get_maconomy_cookie(page).await? {
            return Ok(cookie);
        }

        tokio::time::sleep(POLL_INTERVAL).await;
    }
}
