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

fn assert_snapshot_predicate() -> predicates::function::FnPredicate<impl Fn(&str) -> bool, str> {
    predicates::function::function(move |output: &str| {
        insta::assert_snapshot!(output);
        true
    })
}

#[tokio::main]
#[test]
async fn get_timesheet() {
    // Given
    let mock_server = MockServer::start().await;
    mock_get_instance(None).mount(&mock_server).await;
    mock_get_table_rows(None).mount(&mock_server).await;
    create_test_config();

    // When
    let output = run_json(["get", "--format", "json"], &mock_server.uri());

    // Then
    insta::assert_json_snapshot!(output);
}

#[tokio::main]
#[test]
async fn set_hours() {
    // Given
    let mock_server = MockServer::start().await;
    mock_get_instance(None).mount(&mock_server).await;
    mock_get_table_rows(None).mount(&mock_server).await;
    // These mocks aren't actually required here
    // mock_job_number_search(None).mount(&mock_server).await;
    // mock_tasks_search(None).mount(&mock_server).await;
    mock_add_row(None).mount(&mock_server).await;
    mock_set_hours(None).mount(&mock_server).await;
    create_test_config();

    // When
    let command = [
        "set",
        "8",
        "--job",
        "job one",
        "--task",
        "some task one",
        "--day",
        "monday",
    ];
    let mut output = run(command, &mock_server.uri());

    // Then
    // TODO: try to assert on the values sent to the mock
    output.assert().success();
}

#[tokio::main]
#[test]
async fn set_hours_err() {
    // Given
    let mock_server = MockServer::start().await;
    mock_get_instance(None).mount(&mock_server).await;
    mock_get_table_rows(None).mount(&mock_server).await;
    mock_job_number_search(None).mount(&mock_server).await;
    mock_tasks_search(None).mount(&mock_server).await;
    mock_add_row(None).mount(&mock_server).await;
    create_test_config();

    // When
    let command = [
        "set",
        "--job",
        "doesn't exist",
        "--task",
        "some task one",
        "8",
    ];
    let mut output = run(command, &mock_server.uri());

    // Then
    output
        .assert()
        .stderr(assert_snapshot_predicate())
        .failure();
}
