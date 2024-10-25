use crate::helpers;
use assert_cmd::Command;
use std::{env, ffi, fs::File, io::Write};
use wiremock::MockServer;

const COOKIE_PATH: &str = "./integration_test_auth_cookie";

fn run(
    args: impl IntoIterator<Item = impl AsRef<ffi::OsStr>>,
    server_url: &str,
) -> serde_json::Value {
    env::set_var("MACONOMY__MACONOMY_URL", server_url);
    let output = Command::cargo_bin("maconomy").unwrap().args(args).unwrap();
    serde_json::from_slice(&output.stdout).unwrap()
}

fn use_mock_auth_cookie_file() {
    let cookie = serde_json::json!({
      "name": "Maconomy-mock-cookie",
      "value": "\"mock_cookie_value\""
    });

    let mut file = File::create(COOKIE_PATH).expect("failed to create mock cookie file");
    file.write_all(cookie.to_string().as_bytes())
        .expect("failed to write to mock cookie file");

    env::set_var("MACONOMY__AUTHENTICATION__SSO__COOKIE_PATH", COOKIE_PATH);
}

#[tokio::main]
#[test]
async fn test_get_timesheet() {
    // Given
    let mock_server = MockServer::start().await;
    helpers::maconomy_mock::mock_get_instance(None)
        .mount(&mock_server)
        .await;
    helpers::maconomy_mock::mock_get_table_rows(None)
        .mount(&mock_server)
        .await;
    use_mock_auth_cookie_file();

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

    // When
    let output = run(["get", "--format", "json"], &mock_server.uri());

    // Then
    assert_json_diff::assert_json_include!(
        expected: expected,
        actual: output
    );
}
