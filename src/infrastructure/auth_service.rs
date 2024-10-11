use anyhow::{bail, Context, Result};
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::network::{ClearBrowserCookiesParams, Cookie};
use chromiumoxide::page::Page;
use futures::StreamExt;
use log::{debug, error, info};
use serde::Deserialize;
use std::fmt::Display;
use std::io::{self, BufReader};
use std::{env, fs};
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
}

impl AuthService {
    pub(crate) fn new(login_url: String) -> Self {
        Self {
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

        info!("Cookie file not found, attempting to reauthenticate");
        self.reauthenticate()
            .await
            .context("Failed to reauthenticate")
    }

    pub(crate) async fn reauthenticate(&self) -> Result<AuthCookie> {
        let cookie = self.open_browser_and_authenticate().await?;

        write_cookie_to_file(&cookie)?;

        Ok(cookie.into())
    }

    async fn open_browser_and_authenticate(&self) -> Result<Cookie> {
        let (mut browser, mut handler) = Self::launch_browser(true).await?;
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

        browser.close().await.context("Failed to close browser")?;
        let _ = handle.await;

        Ok(auth_cookie)
    }

    async fn launch_browser(
        with_head: bool,
    ) -> Result<(chromiumoxide::Browser, chromiumoxide::Handler)> {
        let mut builder = BrowserConfig::builder();
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
                panic!("Failed to create browser config. Please make sure that you have either Chromium or Google Chrome installed")
            }
        };

        Browser::launch(browser_config)
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

    pub(crate) async fn logout(&self) -> Result<()> {
        if let Err(err) = fs::remove_file(get_cookie_path()?) {
            if err.kind() != io::ErrorKind::NotFound {
                bail!("Failed to remove auth cookie: {err}");
            }
        };

        self.clear_browser_cookies()
            .await
            .context("Failed to clear browser cookies")
    }
}

fn get_cookie_path() -> Result<String> {
    let home = env::var("HOME")
        .with_context(|| "Could not find home directory. $HOME system variable isn't set")?;
    let cookie_path = format!("{home}/.local/share/maconomy-cli/maconomy_cookie");
    Ok(cookie_path)
}

fn write_cookie_to_file(cookie: &Cookie) -> Result<()> {
    let cookie = serde_json::json!({
        "name": cookie.name,
        "value": cookie.value,
    })
    .to_string();

    let mut file = File::create(get_cookie_path()?).context("Failed to create cookie file")?;
    file.write_all(cookie.as_bytes())
        .context("Failed to write cookie to file")?;

    Ok(())
}

// TODO: change Result to a cleaner type that is NotFound or Other
fn read_cookie_from_file() -> Result<Option<AuthCookie>> {
    let file = match File::open(get_cookie_path()?) {
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
        // TODO: fetch the name of the cookie from the Maconomy-Cookie header, and use that to make
        // sure that we get the right cookie
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
