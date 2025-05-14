use anyhow::Result;
use chrono::{DateTime, Utc};
use reqwest::{
    header::{HeaderMap, HeaderValue},
    RequestBuilder, Response,
};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};
use url::Url;

#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub expires_at: Option<DateTime<Utc>>,
    pub scope: AtprotoScope,
    pub iss: String,
    pub aud: String,
    pub sub: AtprotoDid,
}

impl TokenInfo {
    pub fn is_expired(&self) -> Option<bool> {
        self.expires_at
            .map(|expiry| expiry < (Utc::now() - chrono::Duration::milliseconds(5000)))
    }
}

pub struct OAuthSession {
    pub server: OAuthServerAgent,
    pub sub: AtprotoDid,
    session_getter: SessionGetter,
    dpop_fetch: reqwest::Client,
}

impl OAuthSession {
    pub fn new(server: OAuthServerAgent, sub: AtprotoDid, session_getter: SessionGetter) -> Self {
        let dpop_fetch = dpop_fetch_wrapper(
            reqwest::Client::new(),
            &server.client_metadata.client_id,
            &server.dpop_key,
            &server.server_metadata.dpop_signing_alg_values_supported,
            &server.dpop_nonces,
            false, // is_auth_server
        );

        Self {
            server,
            sub,
            session_getter,
            dpop_fetch,
        }
    }

    pub fn did(&self) -> &AtprotoDid {
        &self.sub
    }

    pub fn server_metadata(&self) -> &OAuthAuthorizationServerMetadata {
        &self.server.server_metadata
    }

    /// Gets the token set, potentially refreshing it.
    ///
    /// # Arguments
    ///
    /// * `refresh` - When `true`, the credentials will be refreshed even if they
    ///   are not expired. When `false`, the credentials will not be refreshed even
    ///   if they are expired. When `auto`, the credentials will be refreshed
    ///   if, and only if, they are (about to be) expired.
    async fn get_token_set(&self, refresh: RefreshMode) -> Result<TokenSet> {
        let no_cache = matches!(refresh, RefreshMode::Force);
        let allow_stale = matches!(refresh, RefreshMode::None);

        let result = self
            .session_getter
            .get(
                &self.sub,
                GetOptions {
                    no_cache,
                    allow_stale,
                },
            )
            .await?;

        Ok(result.token_set)
    }

    /// Gets information about the current token.
    ///
    /// # Arguments
    ///
    /// * `refresh` - Controls when to refresh the token. Defaults to `auto`.
    pub async fn get_token_info(&self, refresh: Option<RefreshMode>) -> Result<TokenInfo> {
        let refresh = refresh.unwrap_or(RefreshMode::Auto);
        let token_set = self.get_token_set(refresh).await?;

        let expires_at = token_set.expires_at.map(|ts| {
            DateTime::<Utc>::from(SystemTime::UNIX_EPOCH + Duration::from_secs(ts as u64))
        });

        Ok(TokenInfo {
            expires_at,
            scope: token_set.scope,
            iss: token_set.iss,
            aud: token_set.aud,
            sub: token_set.sub,
        })
    }

    /// Signs the user out and revokes their token.
    pub async fn sign_out(&self) -> Result<()> {
        let result = self.get_token_set(RefreshMode::None).await;

        match result {
            Ok(token_set) => {
                let _ = self.server.revoke(&token_set.access_token).await;
            }
            Err(_) => {}
        }

        self.session_getter
            .del_stored(
                &self.sub,
                Box::new(TokenRevokedError::new(self.sub.clone())),
            )
            .await?;

        Ok(())
    }

    /// Performs an authenticated fetch to the server.
    pub async fn fetch_handler(
        &self,
        pathname: &str,
        init: Option<RequestInit>,
    ) -> Result<Response> {
        // Try to get a valid token set, refreshing if necessary
        let token_set = self.get_token_set(RefreshMode::Auto).await?;

        let initial_url = Url::parse(&format!("{}{}", token_set.aud, pathname))?;
        let initial_auth = format!("{} {}", token_set.token_type, token_set.access_token);

        let mut request_builder = self.dpop_fetch.request(
            init.as_ref()
                .and_then(|i| i.method.clone())
                .unwrap_or(reqwest::Method::GET),
            initial_url.clone(),
        );

        // Set up headers
        let mut headers = HeaderMap::new();
        if let Some(init_headers) = init.as_ref().and_then(|i| i.headers.clone()) {
            headers.extend(init_headers);
        }
        headers.insert("Authorization", HeaderValue::from_str(&initial_auth)?);
        request_builder = request_builder.headers(headers.clone());

        // Apply other request options
        if let Some(init) = &init {
            if let Some(body) = &init.body {
                request_builder = request_builder.body(body.clone());
            }
        }

        // Make the initial request
        let initial_response = request_builder.send().await?;

        // Check if we got an invalid token error
        if !is_invalid_token_response(&initial_response) {
            return Ok(initial_response);
        }

        // Token was invalid, try to refresh forcefully
        let token_set_fresh = match self.get_token_set(RefreshMode::Force).await {
            Ok(ts) => ts,
            Err(_) => return Ok(initial_response), // Return original response if refresh fails
        };

        // Check if there's a streaming body - if so, we can't retry
        if init
            .as_ref()
            .and_then(|i| i.body.as_ref())
            .map_or(false, |b| b.is_streaming())
        {
            return Ok(initial_response);
        }

        // Prepare a new request with the fresh token
        let final_auth = format!(
            "{} {}",
            token_set_fresh.token_type, token_set_fresh.access_token
        );
        let final_url = Url::parse(&format!("{}{}", token_set_fresh.aud, pathname))?;

        // Update authorization header
        headers.insert("Authorization", HeaderValue::from_str(&final_auth)?);

        // Build and send the final request
        let mut request_builder = self.dpop_fetch.request(
            init.as_ref()
                .and_then(|i| i.method.clone())
                .unwrap_or(reqwest::Method::GET),
            final_url,
        );
        request_builder = request_builder.headers(headers);

        if let Some(init) = &init {
            if let Some(body) = &init.body {
                request_builder = request_builder.body(body.clone());
            }
        }

        let final_response = request_builder.send().await?;

        // If the token is still invalid even after refresh, we have a more serious problem
        if is_invalid_token_response(&final_response) {
            self.session_getter
                .del_stored(
                    &self.sub,
                    Box::new(TokenInvalidError::new(self.sub.clone())),
                )
                .await?;
        }

        Ok(final_response)
    }
}

/// Enum representing different token refresh modes
#[derive(Debug, Clone, Copy)]
pub enum RefreshMode {
    /// Force a refresh regardless of expiration
    Force,
    /// Never refresh, even if expired
    None,
    /// Refresh only if the token is expired or about to expire
    Auto,
}

/// Options for getting a session
#[derive(Debug, Clone)]
pub struct GetOptions {
    pub no_cache: bool,
    pub allow_stale: bool,
}

/// Request initialization options
#[derive(Debug, Clone)]
pub struct RequestInit {
    pub method: Option<reqwest::Method>,
    pub headers: Option<HeaderMap>,
    pub body: Option<RequestBody>,
}

/// Request body types that can be sent
#[derive(Debug, Clone)]
pub enum RequestBody {
    Text(String),
    Json(serde_json::Value),
    Binary(Vec<u8>),
    Stream(Box<dyn futures::io::AsyncRead + Send + Sync>),
}

impl RequestBody {
    fn is_streaming(&self) -> bool {
        matches!(self, RequestBody::Stream(_))
    }
}

/// HTTP Authorization Server Metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthAuthorizationServerMetadata {
    pub dpop_signing_alg_values_supported: Vec<String>,
    // Add other fields as needed
}

/// Checks if the response contains an invalid token error.
///
/// # Arguments
///
/// * `response` - The HTTP response to check
fn is_invalid_token_response(response: &Response) -> bool {
    if response.status() != reqwest::StatusCode::UNAUTHORIZED {
        return false;
    }

    response
        .headers()
        .get("WWW-Authenticate")
        .and_then(|value| value.to_str().ok())
        .map_or(false, |www_auth| {
            (www_auth.starts_with("Bearer ") || www_auth.starts_with("DPoP "))
                && www_auth.contains(r#"error="invalid_token""#)
        })
}
