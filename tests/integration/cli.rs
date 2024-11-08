use crate::helpers::maconomy_mock::{
    mock_add_row, mock_get_instance, mock_get_table_rows, mock_job_number_search, mock_set_hours,
    mock_tasks_search,
};
use assert_cmd::{assert::OutputAssertExt, Command};
use std::{env, ffi};
use wiremock::MockServer;

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
    // These mocks aren't actually required
    // mock_job_number_search(None).mount(&mock_server).await;
    // mock_tasks_search(None).mount(&mock_server).await;
    mock_add_row(None).mount(&mock_server).await;
    mock_set_hours(None).mount(&mock_server).await;
    use_mock_auth_cookie_file();

    // When
    let output = run(
        [
            "set",
            "8",
            "--job",
            "job one",
            "--task",
            "some task one",
            "--day",
            "monday",
        ],
        &mock_server.uri(),
    );

    // Then
    assert!(output.status.success());
}

#[tokio::main]
#[test]
#[ignore]
async fn test_set_hours_err() {
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
        ["set", "--job", "doesn't exist", "--task", "some task", "8"],
        &mock_server.uri(),
    );

    // Then
    // TODO: `output.assert().failure()` doesn't seem to work. Is it because my program panics?
    // dbg!(&output.stdout.into_output().to_string());
    output.assert().failure();
}
