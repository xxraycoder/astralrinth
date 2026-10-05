use crate::event::emit::emit_info;
use crate::state::{Credentials, MinecraftProfile};
use crate::util::astralrinth::utils::{
    get_authlib_injector_library, install_authlib_injector_library,
    install_latest_authlib_injector_library,
    local_authlib_injector_libraries, select_local_authlib_injector_library,
};
use crate::{Result, State};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tokio::process::Command;
use url::Url;
use uuid::Uuid;

const EXTERNAL_AUTH_PROVIDERS: &[ExternalAuthProvider] = &[
    ExternalAuthProvider {
        id: "elyby",
        display_name: "Ely.by",
        icon: "elyby",
        skin_management_url: Some("https://ely.by/skins"),
        authentication: OAuthAuthentication {
            client_id: "astralrinth",
            device_authorization_url: "https://account.ely.by/api/oauth2/v1/devicecode",
            token_url: "https://account.ely.by/api/oauth2/v1/token",
            profile_url: "https://account.ely.by/api/account/v1/info",
            validation_url: "https://authserver.ely.by/auth/validate",
            scope: "account_info offline_access minecraft_server_session",
        },
        launch: ExternalLaunchMethod::AuthlibInjector(ExternalAuthLibrary {
            cache_directory: "elyby",
            release_url: "https://api.xorison.dev/v1/public/lib/elyby/injector",
            server: "ely.by",
        }),
    },
];

pub const OFFLINE_ACCOUNT_TYPE: &str = "offline";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExternalAuthProvider {
    pub id: &'static str,
    pub display_name: &'static str,
    pub icon: &'static str,
    pub skin_management_url: Option<&'static str>,
    authentication: OAuthAuthentication,
    launch: ExternalLaunchMethod,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExternalAuthLibrary {
    pub cache_directory: &'static str,
    pub release_url: &'static str,
    pub server: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct OAuthAuthentication {
    client_id: &'static str,
    device_authorization_url: &'static str,
    token_url: &'static str,
    profile_url: &'static str,
    validation_url: &'static str,
    scope: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ExternalLaunchMethod {
    AuthlibInjector(ExternalAuthLibrary),
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalAuthProviderMetadata {
    pub id: &'static str,
    pub display_name: &'static str,
    pub icon: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skin_management_url: Option<&'static str>,
    pub library_release_url: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalAuthLibraryState {
    pub provider_id: String,
    pub selected_asset_name: Option<String>,
    pub local_asset_names: Vec<String>,
}

pub struct ExternalOAuthFlow {
    pub verification_url: String,
    pub expires_in: i64,
    pub interval: u64,
    device_code: String,
    provider: ExternalAuthProvider,
}

pub enum ExternalOAuthPoll {
    Pending,
    SlowDown,
    Authorized(Credentials),
    Denied,
    Expired,
}

impl ExternalAuthProvider {
    /// Builds the frontend-safe metadata for this provider.
    pub const fn metadata(self) -> ExternalAuthProviderMetadata {
        ExternalAuthProviderMetadata {
            id: self.id,
            display_name: self.display_name,
            icon: self.icon,
            skin_management_url: self.skin_management_url,
            library_release_url: self.library().release_url,
        }
    }

    /// Returns the launcher library configuration selected by this provider.
    pub const fn library(self) -> ExternalAuthLibrary {
        match self.launch {
            ExternalLaunchMethod::AuthlibInjector(library) => library,
        }
    }

    /// Checks whether a provider token can still be used to launch Minecraft.
    async fn validate_access_token(self, access_token: &str) -> Result<bool> {
        validate_yggdrasil_access_token(
            self.authentication.validation_url,
            access_token,
        )
        .await
    }
}

impl OAuthAuthentication {
    /// Requests a device code and prepares the browser verification flow.
    async fn begin(
        self,
        provider: ExternalAuthProvider,
    ) -> Result<ExternalOAuthFlow> {
        let response = reqwest::Client::new()
            .post(self.device_authorization_url)
            .header("Accept", "application/json")
            .form(&HashMap::from([
                ("client_id", self.client_id),
                ("scope", self.scope),
            ]))
            .timeout(Duration::from_secs(15))
            .send()
            .await?;
        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            return Err(oauth_response_error(status, &body));
        }

        let response =
            serde_json::from_str::<DeviceAuthorizationResponse>(&body)?;
        if response.expires_in <= 0 {
            return Err(other_error(
                "OAuth device code lifetime must be positive",
            ));
        }

        Ok(ExternalOAuthFlow {
            verification_url: response.verification_url()?,
            expires_in: response.expires_in,
            interval: response.interval.max(1),
            device_code: response.device_code,
            provider,
        })
    }

    /// Polls the OAuth token endpoint for the current device-code state.
    async fn poll_device_token(
        self,
        device_code: &str,
    ) -> Result<DeviceTokenPoll> {
        let form = HashMap::from([
            ("client_id", self.client_id),
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ("device_code", device_code),
        ]);
        let response = reqwest::Client::new()
            .post(self.token_url)
            .header("Accept", "application/json")
            .form(&form)
            .timeout(Duration::from_secs(15))
            .send()
            .await?;
        let status = response.status();
        let body = response.text().await?;

        if status.is_success() {
            return Ok(DeviceTokenPoll::Authorized(serde_json::from_str(
                &body,
            )?));
        }

        let error = serde_json::from_str::<OAuthErrorResponse>(&body)
            .map_err(|_| oauth_response_error(status, &body))?;
        let error_code = error.error.clone();
        match error_code.as_str() {
            "authorization_pending" => Ok(DeviceTokenPoll::Pending),
            "slow_down" => Ok(DeviceTokenPoll::SlowDown),
            "access_denied" => Ok(DeviceTokenPoll::Denied),
            "expired_token" => Ok(DeviceTokenPoll::Expired),
            _ => Err(other_error(error.message())),
        }
    }

    /// Exchanges a stored refresh token for current OAuth credentials.
    async fn refresh(self, refresh_token: &str) -> Result<OAuthTokenResponse> {
        let form = HashMap::from([
            ("client_id", self.client_id),
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("scope", self.scope),
        ]);

        self.request_token(&form).await
    }

    /// Sends a form-encoded OAuth token request and decodes its response.
    async fn request_token(
        self,
        form: &HashMap<&str, &str>,
    ) -> Result<OAuthTokenResponse> {
        let response = reqwest::Client::new()
            .post(self.token_url)
            .header("Accept", "application/json")
            .form(form)
            .timeout(Duration::from_secs(15))
            .send()
            .await?;
        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            return Err(oauth_response_error(status, &body));
        }

        serde_json::from_str(&body).map_err(Into::into)
    }

    /// Retrieves the Minecraft profile associated with an OAuth access token.
    async fn profile(self, access_token: &str) -> Result<OAuthProfile> {
        let response = reqwest::Client::new()
            .get(self.profile_url)
            .bearer_auth(access_token)
            .header("Accept", "application/json")
            .timeout(Duration::from_secs(15))
            .send()
            .await?;
        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            return Err(other_error(format!(
                "OAuth profile endpoint returned {status}",
            )));
        }

        serde_json::from_str(&body).map_err(Into::into)
    }
}

/// Validates an external-provider access token against its Yggdrasil endpoint.
#[tracing::instrument(skip(access_token))]
async fn validate_yggdrasil_access_token(
    validation_url: &str,
    access_token: &str,
) -> Result<bool> {
    let response = reqwest::Client::new()
        .post(validation_url)
        .header("Content-Type", "application/json")
        .json(&YggdrasilValidateRequest { access_token })
        .timeout(Duration::from_secs(15))
        .send()
        .await?;

    match classify_token_validation_status(response.status()) {
        TokenValidationStatus::Valid => Ok(true),
        TokenValidationStatus::Invalid => Ok(false),
        TokenValidationStatus::Unexpected => Err(other_error(format!(
            "Unexpected access token validation status: {}",
            response.status(),
        ))),
    }
}

/// Maps Yggdrasil HTTP statuses to the states needed by the launcher.
fn classify_token_validation_status(
    status: reqwest::StatusCode,
) -> TokenValidationStatus {
    if status.is_success() {
        TokenValidationStatus::Valid
    } else if matches!(
        status,
        reqwest::StatusCode::BAD_REQUEST | reqwest::StatusCode::UNAUTHORIZED
    ) {
        TokenValidationStatus::Invalid
    } else {
        TokenValidationStatus::Unexpected
    }
}

/// Returns the registry of external account providers supported by AstralRinth.
pub fn external_auth_providers() -> &'static [ExternalAuthProvider] {
    EXTERNAL_AUTH_PROVIDERS
}

/// Looks up a configured provider by its persisted account-type identifier.
pub fn external_auth_provider(id: &str) -> Option<ExternalAuthProvider> {
    external_auth_providers()
        .iter()
        .copied()
        .find(|provider| provider.id == id)
}

/// Returns persisted selections and locally available libraries for every provider.
pub async fn external_auth_library_states() -> Result<Vec<ExternalAuthLibraryState>> {
    let state = State::get().await?;
    let mut selections = sqlx::query_as::<_, (String, String)>(
        "SELECT provider_id, asset_name FROM external_auth_libraries ORDER BY provider_id",
    )
    .fetch_all(&state.pool)
    .await?
    .into_iter()
    .collect::<HashMap<_, _>>();
    let mut libraries = Vec::new();

    for provider in external_auth_providers().iter().copied() {
        libraries.push(ExternalAuthLibraryState {
            provider_id: provider.id.to_string(),
            selected_asset_name: selections.remove(provider.id),
            local_asset_names: local_authlib_injector_libraries(
                provider.library(),
            )
            .await?,
        });
    }

    Ok(libraries)
}

/// Installs and selects one provider library asset after validating it remotely.
pub async fn install_external_auth_library_version(
    provider_id: &str,
    asset_name: &str,
) -> Result<()> {
    let provider = require_external_auth_provider(provider_id)?;
    install_authlib_injector_library(
        provider.id,
        provider.library(),
        asset_name,
    )
    .await
}

/// Selects an already-downloaded provider library version.
pub async fn select_external_auth_library_version(
    provider_id: &str,
    asset_name: &str,
) -> Result<bool> {
    let provider = require_external_auth_provider(provider_id)?;
    select_local_authlib_injector_library(
        provider.id,
        provider.library(),
        asset_name,
    )
    .await
}

/// Resolves a provider or returns an error suitable for API callers.
fn require_external_auth_provider(
    provider_id: &str,
) -> Result<ExternalAuthProvider> {
    external_auth_provider(provider_id).ok_or_else(|| {
        other_error(format!(
            "Unknown external authentication provider: {provider_id}",
        ))
    })
}

/// Starts the provider's OAuth device flow after validating its identifier.
pub async fn begin_external_authentication(
    provider_id: &str,
) -> Result<ExternalOAuthFlow> {
    let provider = require_external_auth_provider(provider_id)?;
    provider.authentication.begin(provider).await
}

/// Advances an OAuth device flow and persists the account after authorization.
pub async fn poll_external_authentication(
    flow: &ExternalOAuthFlow,
) -> Result<ExternalOAuthPoll> {
    let token = match flow
        .provider
        .authentication
        .poll_device_token(&flow.device_code)
        .await?
    {
        DeviceTokenPoll::Pending => return Ok(ExternalOAuthPoll::Pending),
        DeviceTokenPoll::SlowDown => return Ok(ExternalOAuthPoll::SlowDown),
        DeviceTokenPoll::Denied => return Ok(ExternalOAuthPoll::Denied),
        DeviceTokenPoll::Expired => return Ok(ExternalOAuthPoll::Expired),
        DeviceTokenPoll::Authorized(token) => token,
    };
    let refresh_token = token
        .refresh_token
        .as_deref()
        .filter(|token| !token.is_empty())
        .unwrap_or("null");
    let profile = flow
        .provider
        .authentication
        .profile(&token.access_token)
        .await?;
    let state = State::get().await?;

    let credentials = persist_external_account(
        flow.provider,
        profile.uuid,
        &profile.username,
        &token.access_token,
        refresh_token,
        token_expiration(token.expires_in)?,
        &state.pool,
    )
    .await?;

    Ok(ExternalOAuthPoll::Authorized(credentials))
}

/// Refreshes expired credentials for a registered external account, when possible.
pub async fn refresh_external_credentials(
    credentials: &mut Credentials,
) -> Result<bool> {
    let Some(provider) = external_auth_provider(&credentials.account_type)
    else {
        return Ok(false);
    };
    if credentials.refresh_token == "null"
        || credentials.refresh_token.is_empty()
    {
        return Ok(false);
    }

    let token = provider
        .authentication
        .refresh(&credentials.refresh_token)
        .await?;
    credentials.access_token = token.access_token;
    if let Some(refresh_token) =
        token.refresh_token.filter(|token| !token.is_empty())
    {
        credentials.refresh_token = refresh_token;
    }
    credentials.expires = token_expiration(token.expires_in)?;

    Ok(true)
}

/// Stores OAuth-backed credentials in the shared Minecraft user database.
async fn persist_external_account(
    provider: ExternalAuthProvider,
    uuid: Uuid,
    username: &str,
    access_token: &str,
    refresh_token: &str,
    expires: DateTime<Utc>,
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite> + Copy,
) -> Result<Credentials> {
    let credentials = Credentials {
        offline_profile: MinecraftProfile {
            id: uuid,
            name: username.to_string(),
            ..MinecraftProfile::default()
        },
        access_token: access_token.to_string(),
        refresh_token: refresh_token.to_string(),
        expires,
        active: true,
        account_type: provider.id.to_string(),
    };

    credentials.upsert(exec).await?;
    Ok(credentials)
}

/// Applies account-specific JVM launch settings before Minecraft starts.
pub async fn configure_minecraft_launch(
    command: &mut Command,
    credentials: &Credentials,
    version_jar: &str,
) -> Result<()> {
    if credentials.account_type == OFFLINE_ACCOUNT_TYPE {
        configure_offline_launch(command, version_jar).await;
    } else if let Some(provider) =
        external_auth_provider(&credentials.account_type)
    {
        configure_external_launch(command, provider, credentials, version_jar)
            .await?;
    }

    Ok(())
}

/// Restores multiplayer endpoints for offline accounts on affected vanilla versions.
async fn configure_offline_launch(command: &mut Command, version_jar: &str) {
    if version_jar != "1.16.4" && version_jar != "1.16.5" {
        return;
    }

    let invalid_url = "https://invalid.invalid";
    let _ = emit_info(&format!(
		"[AR] Detected launch of {version_jar} on the offline account. Applying vanilla 1.16.4/5 multiplayer fixes.",
	))
	.await;
    command.arg("-Dminecraft.api.env=custom");
    command.arg(format!("-Dminecraft.api.auth.host={invalid_url}"));
    command.arg(format!("-Dminecraft.api.account.host={invalid_url}"));
    command.arg(format!("-Dminecraft.api.session.host={invalid_url}"));
    command.arg(format!("-Dminecraft.api.services.host={invalid_url}"));
}

/// Validates an external account and injects its provider library into Minecraft.
async fn configure_external_launch(
    command: &mut Command,
    provider: ExternalAuthProvider,
    credentials: &Credentials,
    version_jar: &str,
) -> Result<()> {
    let library = provider.library();
    let path = match get_authlib_injector_library(
        provider.id,
        provider.display_name,
        library,
    )
    .await
    {
        Ok(path) => path,
        Err(error) => {
            if !matches!(
                error.raw.as_ref(),
                crate::ErrorKind::ExternalAuthLibraryNotInstalled { .. }
            ) {
                return Err(error);
            }

            if !local_authlib_injector_libraries(library)
                .await?
                .is_empty()
            {
                return Err(error);
            }

            install_latest_authlib_injector_library(provider.id, library)
                .await?
        }
    };

    if !provider
        .validate_access_token(&credentials.access_token)
        .await?
    {
        return Err(other_error(format!(
            "The {} access token is invalid or expired",
            provider.display_name,
        )));
    }

    let _ = emit_info(&format!(
		"[AR] Detected launch of {version_jar} on the {} account. Loading provider library...",
		provider.display_name,
	))
	.await;
    let _ = emit_info(&format!(
        "[AR] Launching Minecraft instance with {}",
        path.display(),
    ))
    .await;
    command.arg(format!("-javaagent:{}={}", path.display(), library.server));

    Ok(())
}

/// Adds a device-flow user code when the provider omits a complete URL.
fn complete_verification_url(
    verification_uri: &str,
    user_code: &str,
) -> Result<String> {
    let mut url = Url::parse(&secure_verification_url(verification_uri)?)
        .map_err(|error| {
            other_error(format!("Invalid OAuth verification URL: {error}"))
        })?;
    url.query_pairs_mut().append_pair("user_code", user_code);

    Ok(url.into())
}

/// Upgrades non-local OAuth verification URLs to HTTPS before opening them.
fn secure_verification_url(verification_uri: &str) -> Result<String> {
    let mut url = Url::parse(verification_uri).map_err(|error| {
        other_error(format!("Invalid OAuth verification URL: {error}"))
    })?;
    if url.scheme() == "http" && !url.host().is_some_and(|host| {
        matches!(host, url::Host::Ipv4(address) if address.is_loopback())
            || matches!(host, url::Host::Ipv6(address) if address.is_loopback())
            || matches!(host, url::Host::Domain("localhost"))
    }) {
        url.set_scheme("https").map_err(|_| {
            other_error("Invalid OAuth verification URL scheme")
        })?;
    }

    Ok(url.into())
}

/// Calculates token expiry, treating providers without a lifetime as non-expiring.
fn token_expiration(expires_in: Option<i64>) -> Result<DateTime<Utc>> {
    let Some(expires_in) = expires_in else {
        return Ok(Utc::now() + ChronoDuration::days(365 * 99));
    };
    if expires_in <= 0 {
        return Err(other_error("OAuth token lifetime must be positive"));
    }

    Ok(Utc::now() + ChronoDuration::seconds(expires_in))
}

/// Extracts a readable error message from a failed OAuth endpoint response.
fn oauth_response_error(
    status: reqwest::StatusCode,
    body: &str,
) -> crate::Error {
    let message = serde_json::from_str::<OAuthErrorResponse>(body)
        .map(OAuthErrorResponse::message)
        .unwrap_or_else(|_| format!("OAuth endpoint returned {status}"));
    other_error(message)
}

/// Creates a domain error for an external-authentication failure.
fn other_error(message: impl Into<String>) -> crate::Error {
    crate::ErrorKind::OtherError(message.into()).as_error()
}

#[derive(Deserialize)]
struct OAuthTokenResponse {
    access_token: String,
    #[serde(default)]
    refresh_token: Option<String>,
    #[serde(default)]
    expires_in: Option<i64>,
}

#[derive(Deserialize)]
struct OAuthErrorResponse {
    error: String,
    #[serde(default)]
    error_description: Option<String>,
    #[serde(default)]
    message: Option<String>,
}

impl OAuthErrorResponse {
    /// Prefers provider detail over the OAuth error code for user-facing failures.
    fn message(self) -> String {
        self.error_description
            .or(self.message)
            .unwrap_or(self.error)
    }
}

#[derive(Deserialize)]
struct DeviceAuthorizationResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    #[serde(default)]
    verification_uri_complete: Option<String>,
    expires_in: i64,
    interval: u64,
}

impl DeviceAuthorizationResponse {
    /// Uses the provider URL or composes one from the verification URI and code.
    fn verification_url(&self) -> Result<String> {
        if let Some(url) = &self.verification_uri_complete {
            return secure_verification_url(url);
        }

        complete_verification_url(&self.verification_uri, &self.user_code)
    }
}

enum DeviceTokenPoll {
    Pending,
    SlowDown,
    Authorized(OAuthTokenResponse),
    Denied,
    Expired,
}

#[derive(Deserialize)]
struct OAuthProfile {
    uuid: Uuid,
    username: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct YggdrasilValidateRequest<'a> {
    access_token: &'a str,
}

#[derive(Debug, Eq, PartialEq)]
enum TokenValidationStatus {
    Valid,
    Invalid,
    Unexpected,
}

#[cfg(test)]
#[path = "authentication/tests.rs"]
mod tests;
