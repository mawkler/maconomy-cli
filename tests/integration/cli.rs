use crate::helpers::maconomy_mock::{
    mock_add_row, mock_get_instance, mock_get_table_rows, mock_job_number_search, mock_set_hours,
    mock_tasks_search,
};
use assert_cmd::Command;
use std::{env, ffi};
use wiremock::{matchers::method, MockServer, ResponseTemplate};

const COOKIE_PATH: &str = "tests/integration/helpers/integration_test_maconomy_cookie";

fn run_json(
    args: impl IntoIterator<Item = impl AsRef<ffi::OsStr>>,
    server_url: &str,
) -> serde_json::Value {
    let output = run(args, server_url);
    serde_json::from_slice(&output.stdout).unwrap()
}

fn run(
    args: impl IntoIterator<Item = impl AsRef<ffi::OsStr>>,
    server_url: &str,
) -> std::process::Output {
    env::set_var("MACONOMY__MACONOMY_URL", server_url);
    Command::cargo_bin("maconomy").unwrap().args(args).unwrap()
    // output.stdout.into_output().to_string()
}

fn use_mock_auth_cookie_file() {
    env::set_var("MACONOMY__AUTHENTICATION__SSO__COOKIE_PATH", COOKIE_PATH);
}

#[tokio::main]
#[test]
async fn test_get_timesheet() {
    // Given
    let mock_server = MockServer::start().await;
    mock_get_instance(None).mount(&mock_server).await;
    mock_get_table_rows(None).mount(&mock_server).await;
    use_mock_auth_cookie_file();

    let expected = serde_json::json!({
      "lines": [
        {
          "job": "Job One",
          "task": "Some task one",
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
          "job": "Job One",
          "task": "Some task two",
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

    // When
    let output = run_json(["get", "--format", "json"], &mock_server.uri());

    // Then
    assert_json_diff::assert_json_include!(
        expected: expected,
        actual: output
    );
}

#[tokio::main]
#[test]
async fn test_set_hours() {
    // Given
    let mock_server = MockServer::start().await;
    mock_get_instance(None).mount(&mock_server).await;
    mock_get_table_rows(None).mount(&mock_server).await;
    mock_job_number_search(None).mount(&mock_server).await;
    mock_set_hours(None).mount(&mock_server).await;
    mock_tasks_search(None).mount(&mock_server).await;
    mock_add_row(None).mount(&mock_server).await;
    use_mock_auth_cookie_file();

    // When
    let output = run(
        ["set", "--job", "job one", "--task", "some task one", "8"],
        &mock_server.uri(),
    );

    // Then
    assert!(output.status.success());

    // TODO: perhaps also check that that the correct endpoint of the mock was called
}
