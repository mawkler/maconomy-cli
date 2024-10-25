// use crate::maconomy_mock;
use assert_cmd::Command;
use std::{env, ffi};
use wiremock::MockServer;

use crate::helpers;

fn run(
    args: impl IntoIterator<Item = impl AsRef<ffi::OsStr>>,
    server_url: &str,
) -> serde_json::Value {
    env::set_var("MACONOMY_MACONOMY_URL", server_url);
    let output = Command::cargo_bin("maconomy").unwrap().args(args).unwrap();
    serde_json::from_slice(&output.stdout).unwrap()
}

// TODO: deal with authentication
#[tokio::main]
#[test]
async fn test_get_timesheet() {
    // Start a local mock HTTP server on a random port
    let mock_server = MockServer::start().await;
    helpers::maconomy_mock::mock_get_instance(None)
        .mount(&mock_server)
        .await;
    helpers::maconomy_mock::mock_get_table_rows(None)
        .mount(&mock_server)
        .await;

    let output = run(["get", "--format", "json"], &mock_server.uri());

    let expected = serde_json::json!({
      "lines": [
        {
          "job": "Job One",
          "task": "Development",
          "week": {
            "monday": 8.0,
            "tuesday": 0.0,
            "wednesday": 0.0,
            "thursday": 0.0,
            "friday": 0.0,
            "saturday": 0.0,
            "sunday": 0.0
          }
        },
        {
          "job": "Job Two",
          "task": "More development",
          "week": {
            "monday": 0.0,
            "tuesday": 0.0,
            "wednesday": 0.0,
            "thursday": 0.0,
            "friday": 0.0,
            "saturday": 0.0,
            "sunday": 0.0
          }
        }
      ]
    });

    assert_json_diff::assert_json_include!(
        expected: expected,
        actual: output
    );
}
