use super::{http_service::HttpService, time_registration::Meta};
use crate::infrastructure::time_registration::TimeRegistration;
use anyhow::{anyhow, bail, Context, Result};
use log::{debug, info};
use reqwest::{header::HeaderMap, Client};
use serde::Deserialize;

const MACONOMY_CONTAINERS_JSON: &str = "application/vnd.deltek.maconomy.containers+json";

struct ContainerInstance {
    id: String,
    /// Needs to be included in a Maconomy-Concurrency-Control header for each request to Maconomy
    concurrency_control: String,
}

pub struct TimeRegistrationRepository {
    client: Client,
    http_service: HttpService,
    url: String,
    company_name: String,
    authorization_cookie: Option<String>,
    container_instance: Option<ContainerInstance>,
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
            container_instance: None,
        })
    }

    async fn get_container_instance(&self) -> Result<ContainerInstance> {
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

        Ok(ContainerInstance {
            id: container_instance_id,
            concurrency_control,
        })
    }

    pub async fn get_time_registration(&mut self) -> Result<TimeRegistration> {
        if self.container_instance.is_none() {
            info!("Fetching container instance");
            let container_instance = self
                .get_container_instance()
                .await
                .context("Failed to get container instance ID")?;

            self.container_instance = Some(container_instance);
        }

        let container_instance = self
            .container_instance
            .as_ref()
            .expect("Missing container instance");
        let instance_id = &container_instance.id;

        let (url, company) = (&self.url, &self.company_name);
        let url =
            format!("{url}/containers/{company}/timeregistration/instances/{instance_id}/data;any");

        let request = self
            .client
            .post(url)
            .header(
                "Maconomy-Concurrency-Control",
                &container_instance.concurrency_control,
            )
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
