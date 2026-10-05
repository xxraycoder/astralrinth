use super::{
    DeviceAuthorizationResponse, OAuthProfile, TokenValidationStatus,
    YggdrasilValidateRequest, classify_token_validation_status,
    external_auth_provider, external_auth_providers, persist_external_account,
    token_expiration,
};
use chrono::{Duration, Utc};
use sqlx::sqlite::SqlitePoolOptions;
use url::Url;
use uuid::Uuid;

#[test]
fn provider_registry_centralizes_backend_and_frontend_metadata() {
    let providers = external_auth_providers();
    let provider = providers[0];
    let library = provider.library();

    assert_eq!(providers.len(), 1);
    assert_eq!(provider.id, "elyby");
    assert_eq!(provider.display_name, "Ely.by");
    assert_eq!(provider.icon, "elyby");
    assert_eq!(
        provider.skin_management_url,
        Some("https://ely.by/skins"),
    );
    assert_eq!(library.cache_directory, "elyby");
    assert_eq!(library.server, "ely.by");
    assert_eq!(external_auth_provider(provider.id), Some(provider));
    assert_eq!(
        serde_json::to_value(provider.metadata()).unwrap(),
        serde_json::json!({
            "id": "elyby",
            "displayName": "Ely.by",
            "icon": "elyby",
            "skinManagementUrl": "https://ely.by/skins",
            "libraryReleaseUrl": "https://api.xorison.dev/v1/public/lib/elyby/injector",
        }),
    );
}

#[test]
fn unknown_provider_is_not_registered() {
    assert_eq!(external_auth_provider("microsoft"), None);
}

#[test]
fn oauth_provider_uses_public_device_flow_and_minecraft_scopes() {
    let authentication =
        external_auth_provider("elyby").unwrap().authentication;

    assert_eq!(authentication.client_id, "astralrinth");
    assert_eq!(
        authentication.device_authorization_url,
        "https://account.ely.by/api/oauth2/v1/devicecode",
    );
    assert_eq!(
        authentication.token_url,
        "https://account.ely.by/api/oauth2/v1/token",
    );
    assert_eq!(
        authentication.scope,
        "account_info offline_access minecraft_server_session",
    );
}

#[test]
fn device_authorization_response_builds_secure_verification_url() {
    let response = serde_json::from_value::<DeviceAuthorizationResponse>(
        serde_json::json!({
            "device_code": "device-code",
            "user_code": "ABCD-EFGH",
            "verification_uri": "http://account.ely.by/code",
            "expires_in": 600,
            "interval": 5,
        }),
    )
    .unwrap();
    let verification_url = response.verification_url().unwrap();
    let verification_url = Url::parse(&verification_url).unwrap();

    assert_eq!(response.device_code, "device-code");
    assert_eq!(response.expires_in, 600);
    assert_eq!(response.interval, 5);
    assert_eq!(verification_url.scheme(), "https");
    assert_eq!(verification_url.host_str(), Some("account.ely.by"));
    assert_eq!(verification_url.path(), "/code");
    assert_eq!(
        verification_url
            .query_pairs()
            .find(|(key, _)| key == "user_code")
            .map(|(_, value)| value.into_owned()),
        Some("ABCD-EFGH".to_string()),
    );
}

#[test]
fn missing_token_lifetime_is_treated_as_non_expiring() {
    let before = Utc::now() + Duration::days(365 * 98);
    let expires = token_expiration(None).unwrap();

    assert!(expires > before);
}

#[test]
fn oauth_profile_uses_elyby_uuid_and_username() {
    let profile = serde_json::from_value::<OAuthProfile>(serde_json::json!({
        "id": 1,
        "uuid": "ffc8fdc9-5824-509e-8a57-c99b940fb996",
        "username": "Player",
        "registeredAt": 1470566470,
        "profileLink": "https://ely.by/u1",
        "preferredLanguage": "en",
    }))
    .unwrap();

    assert_eq!(
        profile.uuid,
        Uuid::parse_str("ffc8fdc9-5824-509e-8a57-c99b940fb996").unwrap(),
    );
    assert_eq!(profile.username, "Player");
}

#[test]
fn token_validation_request_and_statuses_follow_yggdrasil_contract() {
    let request = serde_json::to_value(YggdrasilValidateRequest {
        access_token: "access-token",
    })
    .unwrap();

    assert_eq!(
        request,
        serde_json::json!({ "accessToken": "access-token" }),
    );
    assert_eq!(
        classify_token_validation_status(reqwest::StatusCode::NO_CONTENT),
        TokenValidationStatus::Valid,
    );
    assert_eq!(
        classify_token_validation_status(reqwest::StatusCode::BAD_REQUEST),
        TokenValidationStatus::Invalid,
    );
    assert_eq!(
        classify_token_validation_status(reqwest::StatusCode::UNAUTHORIZED),
        TokenValidationStatus::Invalid,
    );
    assert_eq!(
        classify_token_validation_status(reqwest::StatusCode::BAD_GATEWAY),
        TokenValidationStatus::Unexpected,
    );
}

#[tokio::test]
async fn external_account_is_persisted_with_oauth_tokens() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::query(
        r#"
		CREATE TABLE minecraft_users (
			uuid TEXT PRIMARY KEY NOT NULL,
			active INTEGER NOT NULL,
			username TEXT NOT NULL,
			access_token TEXT NOT NULL,
			refresh_token TEXT NOT NULL,
			expires INTEGER NOT NULL,
			account_type TEXT NOT NULL
		)
		"#,
    )
    .execute(&pool)
    .await
    .unwrap();
    let provider = external_auth_provider("elyby").unwrap();
    let uuid = Uuid::parse_str("0123456789abcdef0123456789abcdef").unwrap();
    let expires = Utc::now() + Duration::hours(1);

    let credentials = persist_external_account(
        provider,
        uuid,
        "Player",
        "access-token",
        "refresh-token",
        expires,
        &pool,
    )
    .await
    .unwrap();
    let stored = sqlx::query_as::<_, (String, String, String, String, i64, String, i64)>(
		"SELECT uuid, username, access_token, refresh_token, expires, account_type, active FROM minecraft_users",
	)
	.fetch_one(&pool)
	.await
	.unwrap();

    assert_eq!(credentials.account_type, "elyby");
    assert_eq!(credentials.offline_profile.id, uuid);
    assert_eq!(stored.0, uuid.as_hyphenated().to_string());
    assert_eq!(stored.1, "Player");
    assert_eq!(stored.2, "access-token");
    assert_eq!(stored.3, "refresh-token");
    assert_eq!(stored.4, expires.timestamp());
    assert_eq!(stored.5, "elyby");
    assert_eq!(stored.6, 1);
}
