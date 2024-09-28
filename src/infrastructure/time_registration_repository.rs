use super::{http_service::HttpService, time_registration::Meta};
use crate::infrastructure::time_registration::TimeRegistration;
use anyhow::{anyhow, bail, Context, Result};
use log::{debug, info};
use reqwest::{header::HeaderMap, Client};
use serde::Deserialize;

const MACONOMY_CONTAINERS_JSON: &str = "application/vnd.deltek.maconomy.containers+json";

pub struct TimeRegistrationRepository {
    client: Client,
    http_service: HttpService,
    url: String,
    company_name: String,
    authorization_cookie: Option<String>,
    /// Needs to be included in a Maconomy-Concurrency-Control header when sending requests to
    /// maconomy
    concurrency_control: Option<String>,
    container_instance_id: Option<String>,
}

#[derive(Deserialize, Debug)]
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

    pub async fn set_container_instance_id(&mut self) -> Result<()> {
        let (url, company) = (&self.url, &self.company_name);
        let url = format!("{url}/containers/{company}/timeregistration/instances");
        let body = include_str!("request_bodies/time_registration_container.json");

        let request = self
            .client
            .post(&url)
            .header("Content-Type", MACONOMY_CONTAINERS_JSON)
            // Specifies the fields that we want from Maconomy
            .body(body);
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

        debug!("Got concurrency control: {concurrency_control}");
        debug!("Got container instance ID: {container_instance_id}");

        self.concurrency_control = Some(concurrency_control);
        self.container_instance_id = Some(container_instance_id);

        Ok(())
    }

    pub async fn get_time_registration(&mut self) -> Result<TimeRegistration> {
        // TODO: refactor out these into a `get_container_instance_id` function
        if self.container_instance_id.is_none() || self.concurrency_control.is_none() {
            info!("Fetching container instance ID");
            self.set_container_instance_id()
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

        let request = self
            .client
            .post(url)
            .header("Maconomy-Concurrency-Control", concurrency_control)
            .header("Content-length", "0");

        let response = self
            .http_service
            .send_request_with_auth(request)
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
