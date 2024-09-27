use super::auth_service::{AuthCookie, AuthService};
use anyhow::{bail, Context, Result};
use log::debug;
use reqwest::StatusCode;

pub struct HttpService {
    auth_service: AuthService,
}

async fn send_with_cookie(
    request: &reqwest::RequestBuilder,
    auth_cookie: AuthCookie,
) -> Result<reqwest::Response> {
    let request = request
        .try_clone()
        .context("Failed to clone request")?
        .header("Cookie", auth_cookie.to_string())
        .header("Authorization", format!("X-Cookie {}", auth_cookie.name));
    debug!("Sending request {request:?}");
    request.send().await.context("Failed to send request")
}

impl HttpService {
    pub(crate) fn new(auth_service: AuthService) -> Self {
        Self { auth_service }
    }

    pub(crate) async fn send_request_with_auth(
        &self,
        request: reqwest::RequestBuilder,
    ) -> Result<reqwest::Response> {
        let auth_cookie = self.auth_service.authenticate().await?;
        let response = send_with_cookie(&request, auth_cookie).await?;
        let status = response.status();

        if status.is_success() {
            return Ok(response);
        }

        debug!("Got {status} from maconomy");
        if let StatusCode::UNAUTHORIZED = response.status() {
            debug!("Attempting to reauthenticate");
            // Reauthenticate (session cookie may have timed out)
            let auth_cookie = self
                .auth_service
                .reauthenticate()
                .await
                .context("Failed to reauthenticate")?;

            let status = response.status();
            let body = response.text().await?;

            let response = send_with_cookie(&request, auth_cookie).await?;
            if !status.is_success() {
                let body = response.text().await?;
                bail!("Request failed with status {status}: {body}\nrequest: {request:?}")
            }
            Ok(response)
        } else {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "failed to decode request body".to_string());
            bail!("Got {status} went wrong when sending request: {body}")
        }
    }
}
