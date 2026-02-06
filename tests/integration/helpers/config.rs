pub(crate) fn create_test_config(uri: &str) -> String {
    format!(
        r#"
            maconomy_url = "{}"
            company_id = "company123"
            [authentication.sso]
            login_url = "https://some.website.com"
            cookie_path = "tests/integration/helpers/integration_test_maconomy_cookie"
        "#,
        uri
    )
}
