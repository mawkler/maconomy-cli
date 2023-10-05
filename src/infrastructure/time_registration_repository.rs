use anyhow::{anyhow, bail, Result};

#[allow(dead_code)]

pub struct TimeRegistrationRepository {
    url: String,
    company_name: String,
    cookies_path: String,
}

impl TimeRegistrationRepository {
    pub fn new(url: String, company_name: String, cookies_path: String) -> Self {
        Self {
            url,
            company_name,
            cookies_path,
        }
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<String> {
        let (url, company_name) = (&self.url, &self.company_name);
        let url = format!("{url}/auth/{company_name}");
        let client = reqwest::Client::new();

        let response = client
            .get(url)
            .basic_auth(username, Some(password))
            .header("Maconomy-Authentication", "X-Cookie")
            .send()
            .await
            .unwrap();

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

        Ok(cookie)
    }
}
