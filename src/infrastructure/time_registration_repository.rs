use super::{http_service::HttpService, time_registration::Meta};
use crate::infrastructure::time_registration::TimeRegistration;
use anyhow::{anyhow, bail, Context, Result};
use log::{debug, info};
use reqwest::{header::HeaderMap, Client, RequestBuilder};
use serde::Deserialize;
use serde_json::json;

const MACONOMY_JSON_CONTENT_TYPE: &str = "application/vnd.deltek.maconomy.containers+json";

#[derive(Clone)]
struct ContainerInstance {
    id: String,
    /// Needs to be included in a Maconomy-Concurrency-Control header for each request to Maconomy
    pub concurrency_control: String,
}

pub(crate) struct TimeRegistrationRepository {
    client: Client,
    http_service: HttpService,
    url: String,
    company_name: String,
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
        .ok_or_else(|| anyhow!("Failed to extract concurrency control from headers"))
}

impl TimeRegistrationRepository {
    pub fn new(
        url: String,
        company_name: String,
        client: Client,
        http_service: HttpService,
    ) -> Self {
        Self {
            url,
            http_service,
            company_name,
            client,
            container_instance: None,
        }
    }

    fn update_concurrency_control(&mut self, concurrency_control: &str) {
        let container_instance = self
            .container_instance
            .as_ref()
            .expect("container instance should have already been set");

        self.container_instance = Some(ContainerInstance {
            concurrency_control: concurrency_control.to_string(),
            id: container_instance.id.clone(),
        });
    }

    async fn fetch_container_instance(&self) -> Result<ContainerInstance> {
        let (url, company) = (&self.url, &self.company_name);
        let url = format!("{url}/containers/{company}/timeregistration/instances");
        let body = include_str!("request_bodies/time_registration_container.json");

        let request = self
            .client
            .post(&url)
            .header("Content-Type", MACONOMY_JSON_CONTENT_TYPE)
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

    async fn get_container_instance(&mut self) -> Result<ContainerInstance> {
        if self.container_instance.is_none() {
            info!("Fetching container instance");
            let container_instance = self
                .fetch_container_instance()
                .await
                .context("Failed to get container instance")?;

            self.container_instance = Some(container_instance);
        }

        let container_instance = self.container_instance.as_ref().ok_or(anyhow!(
            "Missing container instance even though we just fetched it"
        ))?;
        Ok(container_instance.clone())
    }

    async fn send_request(&mut self, request: RequestBuilder) -> Result<reqwest::Response> {
        let response = self
            .http_service
            .send_request_with_auth(request)
            .await
            .context("Failed to send request")?;

        let concurrency_control = concurrency_control_from_headers(response.headers())?;
        self.update_concurrency_control(concurrency_control);

        Ok(response)
    }

    pub async fn get_time_registration(&mut self) -> Result<TimeRegistration> {
        let ContainerInstance {
            id,
            concurrency_control,
        } = self.get_container_instance().await?;

        let (url, company) = (&self.url, &self.company_name);
        let url = format!("{url}/containers/{company}/timeregistration/instances/{id}/data;any");

        let request = self
            .client
            .post(url)
            .header("Maconomy-Concurrency-Control", concurrency_control)
            .header("Content-length", "0");

        let response = self.send_request(request).await?;

        let status = &response.status();
        if !status.is_success() {
            bail!("Server responded with {status}");
        }

        let time_registration = response.json().await.context("Failed to parse response")?;
        Ok(time_registration)
    }

    pub async fn set_time(&mut self, hours: f32, day: u8, row: u8) -> Result<()> {
        let ContainerInstance {
            id,
            concurrency_control,
        } = self.get_container_instance().await?;

        let (url, company) = (&self.url, &self.company_name);
        let url = format!(
            "{url}/containers/{company}/timeregistration/instances/{id}/data/panes/table/{row}"
        );

        let day = format!("numberday{day}");
        let body = json!({"data": {day: hours}});

        let request = self
            .client
            .post(url)
            .header("Maconomy-Concurrency-Control", concurrency_control)
            .header("Content-Type", MACONOMY_JSON_CONTENT_TYPE)
            .body(body.to_string());

        let response = self.send_request(request).await?;

        let status = &response.status();
        if !status.is_success() {
            bail!("Server responded with {status}");
        }

        Ok(())
    }
}
