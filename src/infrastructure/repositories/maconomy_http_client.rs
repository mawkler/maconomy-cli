use std::collections::HashSet;

use anyhow::{anyhow, bail, Context, Result};
use chrono::NaiveDate;
use log::{debug, info};
use reqwest::{
    header::{HeaderMap, ACCEPT, CONTENT_LENGTH, CONTENT_TYPE, USER_AGENT},
    Client, RequestBuilder,
};
use serde::Deserialize;
use serde_json::json;

use crate::infrastructure::{
    http_service::HttpService,
    models::{
        search_response,
        taskname::ShortTaskName,
        time_registration::{Meta, TimeRegistration},
    },
};

// Header values
const MACONOMY_JSON: &str = "application/vnd.deltek.maconomy.containers+json";
const MACONOMY_JSON_V5: &str = "application/vnd.deltek.maconomy.containers+json; version=5.0";
const MACONOMY_CONCURRENCY_CONTROL: &str = "Maconomy-Concurrency-Control";

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

pub(crate) struct MaconomyHttpClient<'a> {
    client: Client,
    http_service: HttpService<'a>,
    url: String,
    company_name: String,
}

#[derive(Deserialize, Debug)]
struct GetInstancesResponseBody {
    meta: Meta,
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum AddRowError {
    #[error("Week has not been initialized")]
    WeekUninitialized,
    #[error("Something went wrong when adding a new row to the time registration")]
    Unknown(#[from] anyhow::Error),
}

fn concurrency_control_from_headers(headers: &HeaderMap) -> Result<String> {
    headers
        .get("maconomy-concurrency-control")
        .and_then(|header| header.to_str().map(ToString::to_string).ok())
        .ok_or_else(|| anyhow!("Failed to extract concurrency control from headers"))
}

impl MaconomyHttpClient<'_> {
    pub fn new(
        url: String,
        company_name: String,
        client: Client,
        http_service: HttpService,
    ) -> MaconomyHttpClient {
        MaconomyHttpClient {
            url,
            http_service,
            company_name,
            client,
        }
    }

    fn get_container_instance_url(&self, container_instance_id: &str) -> String {
        let (url, company) = (&self.url, &self.company_name);
        format!("{url}/containers/{company}/timeregistration/instances/{container_instance_id}")
    }

    pub(crate) async fn get_container_instance(&self) -> Result<ContainerInstance> {
        let (url, company) = (&self.url, &self.company_name);
        let url = format!("{url}/containers/{company}/timeregistration/instances");
        let body = include_str!("request_bodies/time_registration_container.json");

        let request = self
            .client
            .post(&url)
            .header(CONTENT_TYPE, MACONOMY_JSON)
            // Specifies the fields that we want from Maconomy
            .body(body);
        let response = self.send_request(request).await?;

        let status = &response.status();
        if !status.is_success() {
            bail!("Server responded with {status}");
        }

        let concurrency_control = concurrency_control_from_headers(response.headers())?.to_string();
        let container_instance_id = response
            .json::<GetInstancesResponseBody>()
            .await
            .context("Could not deserialize response to container instance")?
            .meta
            .container_instance_id;

        debug!("Got concurrency control: {concurrency_control}");
        debug!("Got container instance ID: {container_instance_id}");

        Ok(ContainerInstance {
            id: ContainerInstanceId(container_instance_id),
            concurrency_control: concurrency_control.into(),
        })
    }

    pub(crate) async fn create_timesheet(
        &self,
        container_instance: &ContainerInstance,
    ) -> Result<ConcurrencyControl> {
        debug!("Creating timesheet...");
        let instance_url = self.get_container_instance_url(&container_instance.id.0);
        let url = format!("{instance_url}/data/panes/card/0/action;name=createtimesheet");
        let concurrency_control = &container_instance.concurrency_control.0;

        let request = self
            .client
            .post(url)
            .header(ACCEPT, MACONOMY_JSON_V5)
            .header(MACONOMY_CONCURRENCY_CONTROL, concurrency_control)
            .header(CONTENT_LENGTH, 0);

        let response = self.send_request(request).await?;

        let status = &response.status();
        if !status.is_success() {
            bail!("Server responded with {status}");
        }

        let concurrency_control = concurrency_control_from_headers(response.headers())?;

        Ok(concurrency_control.into())
    }

    async fn send_request(&self, request: RequestBuilder) -> Result<reqwest::Response> {
        let request = request.header(USER_AGENT, "Maconomy CLI");
        self.http_service
            .send_request_with_auth(&request)
            .await
            .context("Failed to send request")
    }

    pub async fn get_time_registration(
        &self,
        container_instance: &ContainerInstance,
    ) -> Result<(TimeRegistration, ConcurrencyControl)> {
        let instance_url = self.get_container_instance_url(&container_instance.id.0);
        let url = format!("{instance_url}/data;any");
        let concurrency_control = &container_instance.concurrency_control.0;

        let request = self
            .client
            .post(url)
            .header(MACONOMY_CONCURRENCY_CONTROL, concurrency_control)
            .header(CONTENT_LENGTH, "0");

        let response = self.send_request(request).await?;

        let status = &response.status();
        if !status.is_success() {
            bail!("Server responded with {status}");
        }

        let concurrency_control = concurrency_control_from_headers(response.headers())?;
        let time_registration = response
            .json()
            .await
            .context("Failed to parse response to time registration")?;
        Ok((time_registration, concurrency_control.into()))
    }

    pub async fn set_time(
        &self,
        hours: f32,
        days: &HashSet<u8>,
        row: u8,
        container_instance: &ContainerInstance,
    ) -> Result<(TimeRegistration, ConcurrencyControl)> {
        let concurrency_control = &container_instance.concurrency_control.0;
        let instance_url = self.get_container_instance_url(&container_instance.id.0);
        let url = format!("{instance_url}/data/panes/table/{row}");

        let body = set_days_body_from_days(hours, days);
        debug!("setting set_time body to {body}");

        let request = self
            .client
            .post(url)
            .header(MACONOMY_CONCURRENCY_CONTROL, concurrency_control)
            .header(CONTENT_TYPE, MACONOMY_JSON)
            .body(body.to_string());

        let response = self.send_request(request).await?;
        let concurrency_control = concurrency_control_from_headers(response.headers())?;

        let status = &response.status();
        if !status.is_success() {
            bail!("Server responded with {status}");
        }

        let time_registration = response
            .json()
            .await
            .context("Failed to parse response to time registration")?;

        Ok((time_registration, concurrency_control.into()))
    }

    pub async fn get_job_number_from_name(&self, job_name: &str) -> Result<Option<String>> {
        let (url, company) = (&self.url, &self.company_name);
      // https://me73379-maconomy.deltekfirst.com/maconomy-api/containers/me73379/timeregistration/instances/3bc4d944-e663-4b7b-ad96-2531180e3a7f/data/panes/table/init/search;foreignkey=notblockedjobnumber_jobheader
        let url = format!(
        "{url}/containers/{company}/timeregistration/search/table;foreignkey=notblockedjobnumber_jobheader"
    );

        let restriction = format!(
            "(customernumber = '{job_name}' \
                or jobnumber = '{job_name}' \
                or jobname = '{job_name}' \
                or name1 = '{job_name}')"
        );
        let body = json!({
            "restriction": restriction,
            "fields": ["jobnumber"]
        });

        let request = self
            .client
            .post(url)
            .header(CONTENT_TYPE, MACONOMY_JSON)
            .body(body.to_string());

        let response = self.send_request(request).await?;
        let status = &response.status();
        if !status.is_success() {
            bail!("Server responded with {status}");
        }

        let response_body: search_response::SearchResponse<search_response::Jobs> = response
            .json()
            .await
            .context("Failed to parse response body into SearchResponse with jobs")?;
        let job_number = response_body
            .panes
            .filter
            .records
            .first()
            .map(|record| record.data.jobnumber.clone());

        Ok(job_number)
    }

    pub async fn get_tasks_for_job(
        &self,
        job_number: &str,
    ) -> Result<search_response::SearchResponse<search_response::Tasks>> {
        let (url, company) = (&self.url, &self.company_name);
        let url = format!(
        "{url}/containers/{company}/timeregistration/search/table;foreignkey=taskname_tasklistline"
    );

        let body = json!({
                "data": {
                "jobnumber": job_number
            },
            "fields": ["taskname", "description"]
        });

        let request = self
            .client
            .post(url)
            .header(CONTENT_TYPE, MACONOMY_JSON)
            .body(body.to_string());

        let response = self.send_request(request).await?;

        let status = &response.status();
        if !status.is_success() {
            bail!("Server responded with {status}");
        }
        response
            .json()
            .await
            .context("Failed to parse response body into SearchResponse with tasks")
    }

    pub async fn add_new_row(
        &self,
        job_number: &str,
        task_name: &ShortTaskName,
        container_instance: &ContainerInstance,
    ) -> Result<(TimeRegistration, ConcurrencyControl), AddRowError> {
        let concurrency_control = &container_instance.concurrency_control.0;
        let instance_url = self.get_container_instance_url(&container_instance.id.0);
        let url = format!("{instance_url}/data/panes/table/?row=end");
        let body = json!({
            "data": {
                "jobnumber": job_number,
                "taskname": task_name.0
            }
        });

        let request = self
            .client
            .post(url)
            .header(ACCEPT, MACONOMY_JSON_V5)
            .header(CONTENT_TYPE, MACONOMY_JSON_V5)
            .header(MACONOMY_CONCURRENCY_CONTROL, concurrency_control)
            .body(body.to_string());

        let response = self
            .http_service
            .send_request_with_auth_allow_errors(&request)
            .await?;

        let concurrency_control = concurrency_control_from_headers(response.headers());

        let status = &response.status();
        let response_body = response
            .bytes()
            .await
            .context("Failed to get bytes from response body")?;

        if is_uninitialized_week_error(&response_body).await? {
            info!("Week has not been initialized");
            return Err(AddRowError::WeekUninitialized);
        } else if !status.is_success() {
            return Err(anyhow!("Server responded with {status}").into());
        }

        let time_registration =
            serde_json::from_slice(&response_body).context("Failed to parse response")?;

        Ok((time_registration, concurrency_control?.into()))
    }

    pub(crate) async fn delete_row(
        &self,
        line_number: u8,
        container_instance: &ContainerInstance,
    ) -> Result<(TimeRegistration, ConcurrencyControl)> {
        let id = &container_instance.id.0;
        let concurrency_control = &container_instance.concurrency_control.0;
        let instance_url = self.get_container_instance_url(id);
        let url = format!("{instance_url}/data/panes/table/{line_number}");

        let request = self
            .client
            .delete(url)
            .header(ACCEPT, MACONOMY_JSON_V5)
            .header(CONTENT_TYPE, MACONOMY_JSON_V5)
            .header(MACONOMY_CONCURRENCY_CONTROL, concurrency_control);

        let response = self.send_request(request).await?;

        let status = &response.status();
        if !status.is_success() {
            bail!("Server responded with {status}");
        }

        let concurrency_control = concurrency_control_from_headers(response.headers())?;
        let time_registration = response.json().await.context("Failed to parse response")?;

        Ok((time_registration, concurrency_control.into()))
    }

    // Setting the week also returns its time registration
    pub(crate) async fn set_week(
        &self,
        date: NaiveDate,
        container_instance: &ContainerInstance,
    ) -> Result<(TimeRegistration, ConcurrencyControl)> {
        let concurrency_control = &container_instance.concurrency_control.0;
        let instance_url = self.get_container_instance_url(&container_instance.id.0);
        let url = format!("{instance_url}/data/panes/card/0");

        let body = json!({
            "data": {
                "datevar": date.to_string(),
            }
        });

        let request = self
            .client
            .post(url)
            .header(MACONOMY_CONCURRENCY_CONTROL, concurrency_control)
            .header(CONTENT_TYPE, MACONOMY_JSON)
            .body(body.to_string());

        let response = self.send_request(request).await?;

        let status = &response.status();
        if !status.is_success() {
            bail!("Server responded with {status}");
        }

        let concurrency_control = concurrency_control_from_headers(response.headers())?;
        let time_registration = response
            .json()
            .await
            .context("Failed to parse response to time registration")?;
        Ok((time_registration, concurrency_control.into()))
    }

    pub(crate) async fn submit(
        &self,
        container_instance: &ContainerInstance,
    ) -> Result<ConcurrencyControl> {
        let instance_url = self.get_container_instance_url(&container_instance.id.0);
        let url = format!("{instance_url}/data/panes/card/0/action;name=submittimesheet");
        let concurrency_control = &container_instance.concurrency_control.0;

        let request = self
            .client
            .post(url)
            .header(MACONOMY_CONCURRENCY_CONTROL, concurrency_control)
            .header(ACCEPT, MACONOMY_JSON_V5)
            .header(CONTENT_LENGTH, 0);

        let response = self.send_request(request).await?;

        let concurrency_control = concurrency_control_from_headers(response.headers())?;
        Ok(concurrency_control.into())
    }
}

/// The response we get from maconomy if we try to set a value in the time registration without
/// having initialized the week
async fn is_uninitialized_week_error(response_body: &bytes::Bytes) -> Result<bool> {
    let body: serde_json::Value = serde_json::from_slice(response_body)
        .context("Failed to deserialize response body of 'set' conflict")?;

    Ok(body.get("errorMessage").is_some_and(|msg| {
        msg.as_str()
            .is_some_and(|msg| msg.starts_with("Maconomy system error: "))
    }))
}

fn set_days_body_from_days(hours: f32, days: &HashSet<u8>) -> serde_json::Value {
    let days: serde_json::Map<_, _> = days
        .iter()
        .map(|&day| {
            let key = format!("numberday{day}");
            let value = serde_json::json!(hours);
            (key, value)
        })
        .collect();

    serde_json::json!({ "data": days })
}
