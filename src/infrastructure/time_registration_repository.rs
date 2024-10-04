use super::{http_service::HttpService, time_registration::Meta};
use crate::infrastructure::time_registration::TimeRegistration;
use anyhow::{anyhow, bail, Context, Result};
use log::debug;
use reqwest::{header::HeaderMap, Client, RequestBuilder};
use serde::Deserialize;
use serde_json::json;

const MACONOMY_JSON_CONTENT_TYPE: &str = "application/vnd.deltek.maconomy.containers+json";

#[derive(Clone, Debug)]
pub(crate) struct ConcurrencyControl(pub(crate) String);

impl From<String> for ConcurrencyControl {
    fn from(s: String) -> Self {
        ConcurrencyControl(s.to_string())
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ContainerInstanceId(pub(crate) String);

#[derive(Clone, Debug)]
pub(crate) struct ContainerInstance {
    pub(crate) id: ContainerInstanceId,
    /// Needs to be included in a Maconomy-Concurrency-Control header for each request to Maconomy
    pub(crate) concurrency_control: ConcurrencyControl,
}

pub(crate) struct TimeRegistrationRepository {
    client: Client,
    http_service: HttpService,
    url: String,
    company_name: String,
}

#[derive(Deserialize, Debug)]
struct GetInstancesResponseBody {
    meta: Meta,
}

fn concurrency_control_from_headers(headers: &HeaderMap) -> Result<String> {
    headers
        .get("maconomy-concurrency-control")
        .and_then(|header| header.to_str().map(ToString::to_string).ok())
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
        }
    }

    pub(crate) async fn get_container_instance(&self) -> Result<ContainerInstance> {
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
            id: ContainerInstanceId(container_instance_id),
            concurrency_control: concurrency_control.into(),
        })
    }

    async fn send_request(&self, request: RequestBuilder) -> Result<reqwest::Response> {
        self.http_service
            .send_request_with_auth(request)
            .await
            .context("Failed to send request")
    }

    pub async fn get_time_registration(
        &self,
        container_instance: ContainerInstance,
    ) -> Result<(TimeRegistration, ConcurrencyControl)> {
        let (url, company) = (&self.url, &self.company_name);
        let id = container_instance.id.0;
        let concurrency_control = container_instance.concurrency_control.0;
        let url = format!("{url}/containers/{company}/timeregistration/instances/{id}/data;any");

        let request = self
            .client
            .post(url)
            .header("Maconomy-Concurrency-Control", concurrency_control)
            .header("Content-length", "0");

        let response = self.send_request(request).await?;
        let concurrency_control = concurrency_control_from_headers(response.headers())?;

        let status = &response.status();
        if !status.is_success() {
            bail!("Server responded with {status}");
        }

        let time_registration = response.json().await.context("Failed to parse response")?;
        Ok((time_registration, concurrency_control.into()))
    }

    pub async fn set_time(
        &self,
        hours: f32,
        day: u8,
        row: u8,
        container_instance: ContainerInstance,
    ) -> Result<ConcurrencyControl> {
        let (url, company) = (&self.url, &self.company_name);
        let id = container_instance.id.0;
        let concurrency_control = container_instance.concurrency_control.0;
        let url = format!(
            "{url}/containers/{company}/timeregistration/instances/{id}/data/panes/table/{row}"
        );

        let day = format!("numberday{day}");
        let body = json!({"data": {day: hours}});
        debug!("setting set_time body to {body}");

        let request = self
            .client
            .post(url)
            .header("Maconomy-Concurrency-Control", concurrency_control)
            .header("Content-Type", MACONOMY_JSON_CONTENT_TYPE)
            .body(body.to_string());

        let response = self.send_request(request).await?;
        let concurrency_control = concurrency_control_from_headers(response.headers())?;

        let status = &response.status();
        if !status.is_success() {
            bail!("Server responded with {status}");
        }

        Ok(concurrency_control.into())
    }
}
