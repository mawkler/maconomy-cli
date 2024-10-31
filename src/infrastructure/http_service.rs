use std::rc::Rc;

use super::auth_service::{AuthCookie, AuthService};
use anyhow::{bail, Context, Result};
use log::debug;
use reqwest::{
    header::{AUTHORIZATION, COOKIE},
    StatusCode,
};

pub struct HttpService {
    auth_service: Rc<AuthService>,
}

async fn send_with_cookie(
    request: &reqwest::RequestBuilder,
    auth_cookie: AuthCookie,
) -> Result<reqwest::Response> {
    let request = request
        .try_clone()
        .context("Failed to clone request")?
        .header(COOKIE, auth_cookie.to_string())
        .header(AUTHORIZATION, format!("X-Cookie {}", auth_cookie.name));
    request
        .send()
        .await
        .context("Failed to send authenticated request")
}

impl HttpService {
    pub(crate) fn new(auth_service: Rc<AuthService>) -> Self {
        Self { auth_service }
    }

    async fn send_request_with_auth_retry(
        &self,
        request: &reqwest::RequestBuilder,
    ) -> Result<reqwest::Response> {
        let auth_cookie = self.auth_service.authenticate().await?;
        let response = send_with_cookie(request, auth_cookie).await?;
        let status = response.status();

        debug!("Got status {status} from maconomy");
        if let StatusCode::UNAUTHORIZED = status {
            debug!("Attempting to reauthenticate");
            // Reauthenticate (session cookie may have timed out)
            let auth_cookie = self
                .auth_service
                .reauthenticate()
                .await
                .context("Failed to reauthenticate")?;

            let response = send_with_cookie(request, auth_cookie).await?;

            if let StatusCode::UNAUTHORIZED = response.status() {
                panic!(
                    "Failed to reauthenticate. Try logging out with `maconomy logout`, and \
                    running your previous command again. You'll then get asked to sign in again."
                );
            }

            return Ok(response);
        };

        Ok(response)
    }

    pub(crate) async fn send_request_with_auth(
        &self,
        request: &reqwest::RequestBuilder,
    ) -> Result<reqwest::Response> {
        let response = self.send_request_with_auth_retry(request).await?;
        let status = response.status();

        if !status.is_success() {
            let body = response
                .text()
                .await
                .context("failed to decode request body")?;
            bail!("Got response status '{status}' and the following body from maconomy: {body}")
        }

        Ok(response)
    }

    /// Like `send_request_with_auth`, but doesn't check the status code
    pub(crate) async fn send_request_with_auth_allow_errors(
        &self,
        request: &reqwest::RequestBuilder,
    ) -> Result<reqwest::Response> {
        let response = self.send_request_with_auth_retry(request).await?;
        Ok(response)
    }
}
