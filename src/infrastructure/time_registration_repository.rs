use super::{http_service::HttpService, time_registration::Meta};
use crate::infrastructure::time_registration::TimeRegistration;
use anyhow::{anyhow, bail, Context, Result};
use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, CsrfToken, PkceCodeChallenge, RedirectUrl, Scope,
};
use reqwest::{header::HeaderMap, Client};
use serde::Deserialize;

pub struct TimeRegistrationRepository {
    client: Client,
    http_service: HttpService,
    url: String,
    company_name: String,
    authorization_cookie: Option<String>,
    concurrency_control: Option<String>,
    container_instance_id: Option<String>,
}

#[derive(Deserialize)]
struct GetInstancesResponseBody {
    meta: Meta,
}

fn concurrency_control_from_headers(headers: &HeaderMap) -> Result<&str> {
    headers
        .get("maconomy-concurrency-control")
        .and_then(|c| c.to_str().ok())
        .ok_or(anyhow!("Failed to extract concurrency control"))
}

impl TimeRegistrationRepository {
    pub fn new(url: String, company_name: String, http_service: HttpService) -> Result<Self> {
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            url,
            http_service,
            company_name,
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
            .and_then(|c| c.to_str().ok())
            .ok_or(anyhow!("Failed to get authentication cookie"))?
            .to_string();

        self.authorization_cookie = Some(cookie);
        Ok(())
    }

    pub async fn login_sso(
        &mut self,
        auth_url: String,
        client_id: String,
        tenant_id: String,
    ) -> Result<()> {
        // Create an OAuth2 client
        let client = BasicClient::new(
            ClientId::new(client_id),
            None,
            AuthUrl::new(auth_url)?,
            None,
        )
        .set_redirect_uri(RedirectUrl::new("http://localhost".to_string())?);

        // Generate a PKCE challenge.
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        // Generate the full authorization URL.
        let (auth_url, csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            // Set the desired scopes.
            // .add_scope(Scope::new("read".to_string())) // TODO: I don't think I need any scope?
            // .add_scope(Scope::new("write".to_string()))
            .add_scope(Scope::new("openid".to_string()))
            // Set the PKCE code challenge.
            .set_pkce_challenge(pkce_challenge)
            .url();

        Ok(())
    }

    pub fn logged_in(&self) -> bool {
        self.authorization_cookie.is_some()
    }

    pub async fn get_container_instance_id(&mut self) -> Result<()> {
        let (url, company) = (&self.url, &self.company_name);
        let url = format!("{url}/containers/{company}/timeregistration/instances");

        let request = self.client.post(&url).body("{}");
        let response = self
            .http_service
            .send_request_with_auth(request)
            .await
            .context("Failed to send request")?;

        let status = &response.status();
        if !status.is_success() {
            bail!("Server responded with {status}");
        }

        let concurrency_control = concurrency_control_from_headers(response.headers())?.to_string();
        let container_instance_id = response
            .json::<GetInstancesResponseBody>()
            .await
            .context("Could not deserialize response")?
            .meta
            .container_instance_id;

        dbg!(&concurrency_control);
        dbg!(&container_instance_id);

        self.concurrency_control = Some(concurrency_control);
        self.container_instance_id = Some(container_instance_id);
        Ok(())
    }

    pub async fn get_time_registration(&mut self) -> Result<TimeRegistration> {
        if self.container_instance_id.is_none() || self.concurrency_control.is_none() {
            println!("Fetching container instance ID...");
            self.get_container_instance_id()
                .await
                .context("Failed to get container instance ID")?;
        }

        let container_instance_id = self
            .container_instance_id
            .as_ref()
            .expect("Missing container instance ID");
        let concurrency_control = self
            .concurrency_control
            .as_ref()
            .expect("Missing concurrency control");

        let (url, company) = (&self.url, &self.company_name);
        let url = format!("{url}/containers/{company}/timeregistration/instances/{container_instance_id}/data;any");
        let cookie = self
            .authorization_cookie
            .as_ref()
            .ok_or(anyhow!("Not logged in"))?;
        let authorization = format!("X-Cookie {cookie}");

        let response = self
            .client
            .post(url)
            .header("Authorization", authorization)
            .header("Maconomy-Concurrency-Control", concurrency_control)
            .send()
            .await
            .context("Failed to send request")?;

        let status = &response.status();
        if !status.is_success() {
            bail!("Server responded with {status}");
        }

        let time_registration = response.json().await.context("Failed to parse response")?;
        Ok(time_registration)
    }
}
