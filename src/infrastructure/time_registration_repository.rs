use anyhow::{anyhow, bail, Context, Result};
use reqwest::Client;
use serde::Deserialize;

#[allow(dead_code)]

pub struct TimeRegistrationRepository {
    client: Client,
    url: String,
    company_name: String,
    cookies_path: String,
    authorization_cookie: Option<String>,
    concurrency_control: Option<String>,
    container_instance_id: Option<String>,
}

#[derive(Deserialize)]
struct GetInstancesResponseBody {
    meta: Meta,
}

#[derive(Deserialize)]
struct Meta {
    #[serde(rename = "containerInstanceId")]
    container_instance_id: String,
}

impl TimeRegistrationRepository {
    pub fn new(url: String, company_name: String, cookies_path: String) -> Result<Self> {
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            url,
            company_name,
            cookies_path,
            client,
            authorization_cookie: None,
            concurrency_control: None,
            container_instance_id: None,
        })
    }

    pub async fn login(&mut self, username: &str, password: &str) -> Result<()> {
        let (url, company) = (&self.url, &self.company_name);
        let url = format!("{url}/auth/{company}");

        let response = self
            .client
            .get(url)
            .basic_auth(username, Some(password))
            .header("Maconomy-Authentication", "X-Cookie")
            .send()
            .await
            .context("Failed to send request")?;

        let status = &response.status();
        if !status.is_success() {
            bail!("Server responded with {status}");
        }

        let cookie = response
            .headers()
            .get("maconomy-cookie")
            .map(|c| c.to_str().ok())
            .flatten()
            .ok_or(anyhow!("Failed to get authentication cookie"))
            .map(|c| c.to_string())?;

        self.authorization_cookie = Some(cookie);
        Ok(())
    }

    pub async fn get_container_instance_id(&mut self) -> Result<()> {
        let (url, company) = (&self.url, &self.company_name);
        let url = format!("{url}/containers/{company}/timeregistration/instances");
        let cookie = self
            .authorization_cookie
            .as_ref()
            .ok_or(anyhow!("Not logged in"))?;
        let authorization = format!("X-Cookie {cookie}");

        let response = self
            .client
            .post(url)
            .header("Authorization", authorization)
            .header("Content-Type", "application/json")
            .body("{}")
            .send()
            .await
            .context("Failed to send request")?;

        let status = &response.status();
        if !status.is_success() {
            bail!("Server responded with {status}");
        }

        let concurrency_control = response
            .headers()
            .get("maconomy-concurrency-control")
            .map(|c| c.to_str().ok())
            .flatten()
            .ok_or(anyhow!("Failed concurrency control"))
            .map(|c| c.to_string())?;

        let container_instance_id = response
            .json::<GetInstancesResponseBody>()
            .await
            .context("Could not deserialize response")?
            .meta
            .container_instance_id;

        self.concurrency_control = Some(concurrency_control);
        self.container_instance_id = Some(container_instance_id);

        Ok(())
    }
}
