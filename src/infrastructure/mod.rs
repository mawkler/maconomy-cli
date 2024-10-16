pub(crate) mod auth_service;
pub(crate) mod http_service;
pub(crate) mod repositories {
    pub(crate) mod maconomy_http_client;
    pub(crate) mod time_sheet_repository;
}
pub(crate) mod models {
    pub(super) mod search_response;
    pub(super) mod taskname;
    pub(super) mod time_registration;
}
