use crate::maconomy_mock;
use assert_cmd::Command;
use std::env;
use wiremock::MockServer;

#[tokio::main]
#[test]
async fn test_get_timesheet() {
    // Start a local mock HTTP server on a random port
    let mock_server = MockServer::start().await;

    maconomy_mock::mock_get_instances(None)
        .mount(&mock_server)
        .await;

    maconomy_mock::mock_get_table_rows(None)
        .mount(&mock_server)
        .await;

    env::set_var("MACONOMY_MACONOMY_URL", mock_server.uri());

    let output = Command::cargo_bin("maconomy")
        .unwrap()
        .args(["get", "--format", "json"])
        .unwrap();
    let actual: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

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
        actual: actual
    );
}
