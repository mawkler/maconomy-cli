use super::auth_service::{AuthCookie, AuthService};
use anyhow::{bail, Context, Result};
use reqwest::StatusCode;

pub struct HttpService {
    auth_service: AuthService,
}

async fn send_with_cookie(
    request: &reqwest::RequestBuilder,
    auth_cookie: AuthCookie,
) -> Result<reqwest::Response> {
    request
        .try_clone()
        .context("Failed to clone request")?
        .header("Cookie", auth_cookie.to_string())
        .send()
        .await
        .context("Failed to send request")
}

impl HttpService {
    pub(crate) fn new(auth_service: AuthService) -> Self {
        Self { auth_service }
    }

    async fn send_request_with_auth(
        &self,
        request: reqwest::RequestBuilder,
    ) -> Result<reqwest::Response> {
        let auth_cookie = self.auth_service.authenticate().await?;
        let response = send_with_cookie(&request, auth_cookie).await?;

        if let StatusCode::UNAUTHORIZED = response.status() {
            let auth_cookie = self
                .auth_service
                .reauthenticate()
                .await
                .context("Failed to reauthenticate")?;

            send_with_cookie(&request, auth_cookie)
                .await?
                .error_for_status()
                .context("Failed to send request, even after reauthenticating")
        } else if response.status().is_success() {
            return Ok(response);
        } else {
            bail!("Something went wrong when sending request")
        }
    }
}
