use std::rc::Rc;

use super::auth_service::{AuthCookie, AuthService};
use anyhow::{bail, Context, Result};
use log::{debug, warn};
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

    pub(crate) async fn send_request_with_auth(
        &self,
        request: &reqwest::RequestBuilder,
    ) -> Result<reqwest::Response> {
        let auth_cookie = self.auth_service.authenticate().await?;
        let response = send_with_cookie(request, auth_cookie).await?;
        let status = response.status();

        if status.is_success() {
            return Ok(response);
        }

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

            let status = response.status();
            if status.is_success() {
                Ok(response)
            } else if let StatusCode::UNAUTHORIZED = status {
                panic!(
                    "Failed to reauthenticate. Try logging out with `maconomy logout`, and \
                    running your previous command again."
                );
            } else {
                warn!("Got status code {status}");
                let body = response
                    .text()
                    .await
                    .context("Failed to get body of response")?;
                bail!("Request failed with status {status}: {body}\nrequest: {request:?}");
            }
        } else {
            let body = response
                .text()
                .await
                .context("failed to decode request body")?;
            bail!("Got response body {body}")
        }
    }
}
