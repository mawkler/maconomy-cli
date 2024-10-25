use serde_json::json;
use uuid::Uuid;
use wiremock::matchers::method;

use super::mock_data::{self, get_mock_table_rows_response};

// Regex
const UUID_REGEX: &str = "[a-z0-9-]{36}";
const COMPANY_REGEX: &str = "[a-z0-9]+";
const ROW_NUMBER_REGEX: &str = r"\d+";

// Headers
const MACONOMY_CONCURRENCY_CONTROL: &str = "Maconomy-Concurrency-Control";

fn create_mock(
    path: &str,
    default_body: serde_json::Value,
    response: Option<wiremock::ResponseTemplate>,
) -> wiremock::Mock {
    let default_response = wiremock::ResponseTemplate::new(200)
        .append_header(MACONOMY_CONCURRENCY_CONTROL, Uuid::new_v4().to_string())
        .set_body_json(default_body);
    let response = response.unwrap_or(default_response);

    wiremock::Mock::given(method("POST"))
        .and(wiremock::matchers::path_regex(path))
        .respond_with(response)
}

fn mock_container_instance_body() -> serde_json::Value {
    json!({
        "meta": {
            "containerName": "timeregistration",
            "containerInstanceId": Uuid::new_v4().to_string()
        }
    })
}

pub(crate) fn mock_get_instance(response: Option<wiremock::ResponseTemplate>) -> wiremock::Mock {
    let path_regex = format!("/containers/{COMPANY_REGEX}/timeregistration/instances$");
    let default_body = mock_container_instance_body();

    create_mock(&path_regex, default_body, response)
}

pub(crate) fn mock_get_table_rows(response: Option<wiremock::ResponseTemplate>) -> wiremock::Mock {
    let path_regex =
        format!("/containers/{COMPANY_REGEX}/timeregistration/instances/{UUID_REGEX}/data;any$");
    let default_body = mock_data::get_mock_table_rows_response();

    create_mock(&path_regex, default_body, response)
}

pub(crate) fn mock_job_number_search(
    response: Option<wiremock::ResponseTemplate>,
) -> wiremock::Mock {
    let path_regex = format!(
        "/containers/{COMPANY_REGEX}/timeregistration/search/table;foreignkey=notblockedjobnumber_jobheader"
    );
    let default_body = serde_json::json!({
      "panes": {
        "filter": {
          "meta": {
              "paneName": "filter",
              "rowCount": 25,
              "rowOffset": 0
          },
          "records": [
            {
              "data": {
                "jobnumber": "1234567",
              }
            }
          ]
        }
      }
    });

    create_mock(&path_regex, default_body, response)
}

pub(crate) fn mock_tasks_search(response: Option<wiremock::ResponseTemplate>) -> wiremock::Mock {
    let path_regex = format!(
        "/containers/{COMPANY_REGEX}/timeregistration/search/table;foreignkey=taskname_tasklistline"
    );
    let default_body = serde_json::json!({
      "panes": {
        "filter": {
          "meta": {
              "paneName": "filter",
              "rowCount": 11,
              "rowOffset": 0
          },
          "records": [
            {
              "data": {
                "taskname": "task one",
                "tasklist": "ABC0000001",
                "description": "some task one"
              }
            },
            {
              "data": {
                "taskname": "task two",
                "tasklist": "ABC0000002",
                "description": "some task two"
              }
            },
            {
              "data": {
                "taskname": "task three",
                "tasklist": "ABC0000003",
                "description": "some task three"
              }
            }
          ]
        }
      }
    });

    create_mock(&path_regex, default_body, response)
}

pub(crate) fn mock_add_row(response: Option<wiremock::ResponseTemplate>) -> wiremock::Mock {
    let path_regex = format!(
        r"/containers/{COMPANY_REGEX}/timeregistration/instances/{UUID_REGEX}/data/panes/table/$"
    );
    let default_response = wiremock::ResponseTemplate::new(200)
        .append_header(MACONOMY_CONCURRENCY_CONTROL, Uuid::new_v4().to_string())
        .set_body_json(mock_data::get_mock_table_rows_response());
    let response = response.unwrap_or(default_response);

    wiremock::Mock::given(method("POST"))
        .and(wiremock::matchers::path_regex(path_regex))
        .and(wiremock::matchers::query_param("row", "end"))
        .respond_with(response)
}

pub(crate) fn mock_set_hours(response: Option<wiremock::ResponseTemplate>) -> wiremock::Mock {
    let path_regex = format!(
        "/containers/{COMPANY_REGEX}/timeregistration/instances/{UUID_REGEX}/data/panes/table/{ROW_NUMBER_REGEX}$"
    );
    let default_body = get_mock_table_rows_response();

    create_mock(&path_regex, default_body, response)
}
