use reqwest::Url;

use crate::commands::FALLBACK_BASE_URL;
use crate::escl::EsclClient;

#[test]
fn fallback_base_url_is_valid() {
    let url = Url::parse(FALLBACK_BASE_URL).expect("fallback URL literal is valid");

    assert!(EsclClient::new(url).is_ok());
}
