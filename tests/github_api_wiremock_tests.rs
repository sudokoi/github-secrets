use base64::{Engine, engine::general_purpose};
use github_secrets::github::GitHubClient;
use octocrab::Octocrab;
use wiremock::matchers::{method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_get_secret_info_not_found() {
    let mock_server = MockServer::start().await;

    // Mock the GET for secret to return 404
    Mock::given(method("GET"))
        .and(path_regex(r"/repos/.*/.*/actions/secrets/TEST_SECRET"))
        .respond_with(ResponseTemplate::new(404).set_body_string(r#"{"message":"Not Found"}"#))
        .mount(&mock_server)
        .await;

    // Build Octocrab with mock base URL
    let octocrab = Octocrab::builder()
        .personal_token("test-token".to_string())
        .base_uri(mock_server.uri())
        .unwrap()
        .build()
        .unwrap();

    let client = GitHubClient::with_octocrab(octocrab, "owner".to_string(), "repo".to_string());

    let info = client
        .get_secret_info("TEST_SECRET")
        .await
        .expect("call should succeed");
    assert!(info.is_none(), "Expected None for 404 response");
}

#[tokio::test]
async fn test_update_secret_success_204() {
    let mock_server = MockServer::start().await;

    // Mock GET public-key
    let mut public_key_bytes = [0u8; sodoken::crypto_box::XSALSA_PUBLICKEYBYTES];
    let mut secret_key_bytes = [0u8; sodoken::crypto_box::XSALSA_SECRETKEYBYTES];
    sodoken::crypto_box::xsalsa_keypair(&mut public_key_bytes, &mut secret_key_bytes).unwrap();
    let public_key_b64 = general_purpose::STANDARD.encode(&public_key_bytes);
    let pk_body = format!(r#"{{"key_id":"test-key-id","key":"{}"}}"#, public_key_b64);

    Mock::given(method("GET"))
        .and(path_regex(r"/repos/.*/.*/actions/secrets/public-key"))
        .respond_with(ResponseTemplate::new(200).set_body_string(pk_body))
        .mount(&mock_server)
        .await;

    // Mock PUT secret to return 204 No Content (empty body)
    Mock::given(method("PUT"))
        .and(path_regex(r"/repos/.*/.*/actions/secrets/.*"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    let octocrab = Octocrab::builder()
        .personal_token("test-token".to_string())
        .base_uri(mock_server.uri())
        .unwrap()
        .build()
        .unwrap();

    let client = GitHubClient::with_octocrab(octocrab, "owner".to_string(), "repo".to_string());

    let res = client.update_secret("MY_SECRET", "supersecret").await;
    assert!(
        res.is_ok(),
        "Expected update_secret to succeed on 204 response"
    );
}

#[tokio::test]
async fn test_update_secret_api_error_propagates() {
    let mock_server = MockServer::start().await;

    // Mock GET public-key
    let mut public_key_bytes = [0u8; sodoken::crypto_box::XSALSA_PUBLICKEYBYTES];
    let mut secret_key_bytes = [0u8; sodoken::crypto_box::XSALSA_SECRETKEYBYTES];
    sodoken::crypto_box::xsalsa_keypair(&mut public_key_bytes, &mut secret_key_bytes).unwrap();
    let public_key_b64 = general_purpose::STANDARD.encode(&public_key_bytes);
    let pk_body = format!(r#"{{"key_id":"test-key-id","key":"{}"}}"#, public_key_b64);

    Mock::given(method("GET"))
        .and(path_regex(r"/repos/.*/.*/actions/secrets/public-key"))
        .respond_with(ResponseTemplate::new(200).set_body_string(pk_body))
        .mount(&mock_server)
        .await;

    // Mock PUT secret to return 400 with GitHub-style error body
    let err_body = r#"{"message":"Bad Request","errors":[{"resource":"Secret","code":"custom","message":"Invalid"}],"documentation_url":"https://docs.github.com"}"#;
    Mock::given(method("PUT"))
        .and(path_regex(r"/repos/.*/.*/actions/secrets/.*"))
        .respond_with(ResponseTemplate::new(400).set_body_string(err_body))
        .mount(&mock_server)
        .await;

    let octocrab = Octocrab::builder()
        .personal_token("test-token".to_string())
        .base_uri(mock_server.uri())
        .unwrap()
        .build()
        .unwrap();

    let client = GitHubClient::with_octocrab(octocrab, "owner".to_string(), "repo".to_string());

    let res = client.update_secret("MY_SECRET", "supersecret").await;
    assert!(
        res.is_err(),
        "Expected update_secret to return error on 400 response"
    );
    let msg = res.unwrap_err().to_string();
    assert!(
        msg.contains("GitHub API error") || msg.contains("Bad Request") || msg.contains("Invalid")
    );
}
