use serde_json::json;
use wiremock::matchers::method;

use crate::mock_data;

// TODO: generate UUID dynamically
const UUID: &str = "bdbcbf53-404d-49d9-98f9-597bbc8b283a";

// Regex
const UUID_REGEX: &str = "[a-z0-9-]{36}";
const COMPANY_REGEX: &str = "[a-z0-9]+";

// Headers
const MACONOMY_CONCURRENCY_CONTROL: &str = "Maconomy-Concurrency-Control";

fn create_mock(
    path: &str,
    default_body: serde_json::Value,
    response: Option<wiremock::ResponseTemplate>,
) -> wiremock::Mock {
    let default_response = wiremock::ResponseTemplate::new(200)
        .append_header(MACONOMY_CONCURRENCY_CONTROL, UUID)
        .set_body_json(default_body);
    let response = response.unwrap_or(default_response);

    wiremock::Mock::given(method("POST"))
        .and(wiremock::matchers::path_regex(path))
        .respond_with(response)
}

pub(crate) fn mock_get_instances(response: Option<wiremock::ResponseTemplate>) -> wiremock::Mock {
    let path_regex = format!("/containers/{COMPANY_REGEX}/timeregistration/instances$");
    let default_body = json!({
        "meta": {
            "containerName": "timeregistration",
            "containerInstanceId": UUID
        }
    });

    create_mock(&path_regex, default_body, response)
}

pub(crate) fn mock_get_table_rows(response: Option<wiremock::ResponseTemplate>) -> wiremock::Mock {
    let path_regex =
        format!("/containers/{COMPANY_REGEX}/timeregistration/instances/{UUID_REGEX}/data;any$");
    let default_body = mock_data::get_mock_table_rows_response();

    create_mock(&path_regex, default_body, response)
}
