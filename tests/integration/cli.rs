use crate::helpers::{
    config::create_test_config,
    maconomy_mock::{
        mock_add_row, mock_get_instance, mock_get_table_rows, mock_job_number_search,
        mock_set_hours, mock_tasks_search,
    },
};
use assert_cmd::Command;
use std::{env, ffi};
use wiremock::MockServer;

fn run_json(
    args: impl IntoIterator<Item = impl AsRef<ffi::OsStr>>,
    server_url: &str,
) -> serde_json::Value {
    let output = run(args, server_url).unwrap();
    serde_json::from_slice(&output.stdout).unwrap()
}

fn run(
    args: impl IntoIterator<Item = impl AsRef<ffi::OsStr>>,
    server_url: &str,
) -> assert_cmd::Command {
    env::set_var("MACONOMY__MACONOMY_URL", server_url);
    let mut cmd = Command::cargo_bin("maconomy").unwrap();
    cmd.args(args);
    cmd
}

#[tokio::main]
#[test]
async fn test_get_timesheet() {
    // Given
    let mock_server = MockServer::start().await;
    mock_get_instance(None).mount(&mock_server).await;
    mock_get_table_rows(None).mount(&mock_server).await;
    create_test_config();

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
    create_test_config();

    // When
    let cmd = [
        "set",
        "8",
        "--job",
        "job one",
        "--task",
        "some task one",
        "--day",
        "monday",
    ];
    let mut output = run(cmd, &mock_server.uri());

    // Then
    output.assert().success();
}

#[tokio::main]
#[test]
async fn test_set_hours_err() {
    // Given
    let mock_server = MockServer::start().await;
    mock_get_instance(None).mount(&mock_server).await;
    mock_get_table_rows(None).mount(&mock_server).await;
    mock_job_number_search(None).mount(&mock_server).await;
    mock_tasks_search(None).mount(&mock_server).await;
    mock_add_row(None).mount(&mock_server).await;
    create_test_config();

    // When
    let cmd = [
        "set",
        "--job",
        "doesn't exist",
        "--task",
        "some task one",
        "8",
    ];
    let mut output = run(cmd, &mock_server.uri());

    let expected_stdoud_prefix = "Something went wrong when adding a new line to the time sheet: \
        did not find job 'doesn't exist' and task 'some task one', even after creating a new line \
        for it";
    output
        .assert()
        .stderr(predicates::str::starts_with(expected_stdoud_prefix))
        .failure();
}
