const CONFIG: &[(&str, &str)] = &[
    ("COMPANY_ID", "company123"),
    ("AUTHENTICATION__SSO__LOGIN_URL", "https://some.website.com"),
    (
        "AUTHENTICATION__SSO__COOKIE_PATH",
        "tests/integration/helpers/integration_test_maconomy_cookie",
    ),
    // `MACONOMY_URL` is set by the tests (from mock URL)
];

pub(crate) fn create_test_config() {
    for (key, value) in CONFIG {
        std::env::set_var(format!("MACONOMY__{key}"), value);
    }
}
