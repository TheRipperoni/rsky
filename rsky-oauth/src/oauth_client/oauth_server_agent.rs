use crate::jwk::Keyset;
use crate::oauth_client::atproto_token_response::{AtprotoScope, AtprotoTokenResponse};
use crate::oauth_client::oauth_session::OAuthAuthorizationServerMetadata;
use crate::oauth_types::{OAuthClientMetadata, OAuthParResponse};
use crate::simple_store::SimpleStore;
use anyhow::Result;
use chrono::{DateTime, Utc};
use reqwest::{self, header::HeaderMap};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as Json};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;
use url::Url;

/// Represents a token set with all relevant information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenSet {
    pub iss: String,
    pub sub: AtprotoDid,
    pub aud: String,
    pub scope: AtprotoScope,

    pub refresh_token: Option<String>,
    pub access_token: String,
    pub token_type: String, // Always "DPoP"

    /// ISO Date string
    pub expires_at: Option<String>,
}

/// Cache for DPoP nonces
pub type DpopNonceCache = dyn SimpleStore<String, String, Error = ()>;

/// OAuth Server Agent for interacting with an OAuth server
pub struct OAuthServerAgent {
    pub dpop_key: Key,
    pub server_metadata: OAuthAuthorizationServerMetadata,
    pub client_metadata: OAuthClientMetadata,
    pub dpop_nonces: DpopNonceCache,
    oauth_resolver: Arc<OAuthResolver>,
    runtime: Arc<Runtime>,
    keyset: Option<Keyset>,
    dpop_fetch: Fetch,
}

impl OAuthServerAgent {
    /// Creates a new OAuthServerAgent
    pub fn new(
        dpop_key: Key,
        server_metadata: OAuthAuthorizationServerMetadata,
        client_metadata: ClientMetadata,
        dpop_nonces: DpopNonceCache,
        oauth_resolver: OAuthResolver,
        runtime: Runtime,
        keyset: Option<Keyset>,
        fetch: Option<reqwest::Client>,
    ) -> Self {
        let oauth_resolver = Arc::new(oauth_resolver);
        let runtime = Arc::new(runtime);

        // Create DPoP-enabled fetch wrapper
        let dpop_fetch = dpop_fetch_wrapper(
            fetch.unwrap_or_else(|| reqwest::Client::new()),
            &client_metadata.client_id,
            &dpop_key,
            &server_metadata.dpop_signing_alg_values_supported,
            &runtime,
            &dpop_nonces,
            true, // is_auth_server
        );

        Self {
            dpop_key,
            server_metadata,
            client_metadata,
            dpop_nonces,
            oauth_resolver,
            runtime,
            keyset,
            dpop_fetch,
        }
    }

    /// Returns the issuer URL
    pub fn issuer(&self) -> &str {
        &self.server_metadata.issuer
    }

    /// Revokes a token
    pub async fn revoke(&self, token: &str) -> Result<()> {
        // Attempt to revoke the token but ignore any errors
        let _ = self.request("revocation", json!({ "token": token })).await;
        Ok(())
    }

    /// Exchanges an authorization code for a token set
    pub async fn exchange_code(&self, code: &str, code_verifier: Option<&str>) -> Result<TokenSet> {
        let now = chrono::Utc::now();

        let mut payload = json!({
            "grant_type": "authorization_code",
            "redirect_uri": self.client_metadata.redirect_uris[0],
            "code": code,
        });

        if let Some(verifier) = code_verifier {
            payload["code_verifier"] = json!(verifier);
        }

        let token_response: AtprotoTokenResponse = self.request("token", payload).await?;

        // Verify the issuer before accepting the token
        let aud = match self.verify_issuer(&token_response.sub).await {
            Ok(aud) => aud,
            Err(err) => {
                // Revoke the token if verification fails
                let _ = self.revoke(&token_response.access_token).await;
                return Err(err);
            }
        };

        // Calculate expiration time if expires_in is provided
        let expires_at = token_response
            .expires_in
            .map(|seconds| (now + chrono::Duration::seconds(seconds as i64)).to_rfc3339());

        Ok(TokenSet {
            aud,
            sub: token_response.sub,
            iss: self.issuer().to_string(),

            scope: token_response.scope,
            refresh_token: token_response.refresh_token,
            access_token: token_response.access_token,
            token_type: token_response.token_type,

            expires_at,
        })
    }

    /// Refreshes a token set using the refresh token
    pub async fn refresh(&self, token_set: TokenSet) -> Result<TokenSet> {
        let refresh_token = match &token_set.refresh_token {
            Some(rt) => rt,
            None => {
                return Err(anyhow::anyhow!(TokenRefreshError::new(
                    &token_set.sub,
                    "No refresh token available"
                )))
            }
        };

        // Verify the issuer before attempting to refresh
        let aud = self.verify_issuer(&token_set.sub).await?;

        let now = chrono::Utc::now();

        let payload = json!({
            "grant_type": "refresh_token",
            "refresh_token": refresh_token,
        });

        let token_response: AtprotoTokenResponse = self.request("token", payload).await?;

        // Calculate expiration time if expires_in is provided
        let expires_at = token_response
            .expires_in
            .map(|seconds| (now + chrono::Duration::seconds(seconds as i64)).to_rfc3339());

        Ok(TokenSet {
            aud,
            sub: token_set.sub,
            iss: self.issuer().to_string(),

            scope: token_response.scope,
            refresh_token: token_response.refresh_token,
            access_token: token_response.access_token,
            token_type: token_response.token_type,

            expires_at,
        })
    }

    /// Verifies that the issuer in the DID resolution matches the server's issuer
    ///
    /// This is a critical security step to validate that the sub (DID) is actually
    /// authorized by the expected issuer.
    ///
    /// Returns the user's PDS URL (the resource server for the user)
    async fn verify_issuer(&self, sub: &AtprotoDid) -> Result<String> {
        let signal = timeout_signal(std::time::Duration::from_secs(10));

        let resolved = self
            .oauth_resolver
            .resolve_from_identity(
                sub,
                true,  // no_cache
                false, // allow_stale
                Some(signal),
            )
            .await?;

        if self.issuer() != resolved.metadata.issuer {
            return Err(anyhow::anyhow!("Issuer mismatch"));
        }

        Ok(resolved.identity.pds.href)
    }

    /// Makes a request to an OAuth endpoint
    async fn request<T>(&self, endpoint: &str, payload: serde_json::Value) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let endpoint_url = self.server_metadata.get_endpoint_url(endpoint)?;

        let auth = self.build_client_auth(endpoint).await?;

        // Merge payloads
        let mut request_payload = payload;
        if let Some(auth_payload) = auth.payload {
            for (k, v) in auth_payload.as_object().unwrap() {
                request_payload[k] = v.clone();
            }
        }

        // Build headers
        let mut headers = HeaderMap::new();
        if let Some(auth_headers) = auth.headers {
            for (k, v) in auth_headers {
                headers.insert(k.parse().unwrap(), v.parse().unwrap());
            }
        }
        headers.insert("Content-Type", "application/json".parse().unwrap());

        let response = self
            .dpop_fetch
            .post(endpoint_url)
            .headers(headers)
            .json(&request_payload)
            .send()
            .await?;

        let status = response.status();
        let json: Json = response.json().await?;

        if status.is_success() {
            match endpoint {
                "token" => {
                    // Validate the token response schema
                    let token_response = atproto_token_response_schema(json)?;
                    Ok(token_response)
                }
                "pushed_authorization_request" => {
                    // Validate the PAR response schema
                    let par_response = oauth_par_response_schema(json)?;
                    Ok(par_response)
                }
                _ => {
                    // For other endpoints, just return the JSON as is
                    Ok(serde_json::from_value(json)?)
                }
            }
        } else {
            Err(anyhow::anyhow!(OAuthResponseError::new(
                status.as_u16(),
                json
            )))
        }
    }

    /// Builds authentication information for client requests
    async fn build_client_auth(&self, endpoint: &str) -> Result<ClientAuth> {
        let method_supported = self
            .server_metadata
            .get_token_endpoint_auth_methods_supported();
        let method = self.client_metadata.token_endpoint_auth_method.as_deref();

        // Try private_key_jwt if it's explicitly requested or supported and we have a keyset
        if method == Some("private_key_jwt")
            || (self.keyset.is_some()
                && method.is_none()
                && method_supported.contains(&"private_key_jwt".to_string()))
        {
            if let Some(keyset) = &self.keyset {
                // Get supported algorithms or use fallback
                let alg = self
                    .server_metadata
                    .token_endpoint_auth_signing_alg_values_supported
                    .clone()
                    .unwrap_or_else(|| vec![FALLBACK_ALG.to_string()]);

                // Get key IDs from jwks if available
                let kid = self.client_metadata.jwks.as_ref().map(|jwks| {
                    jwks.keys
                        .iter()
                        .filter_map(|key| key.kid.clone())
                        .collect::<Vec<String>>()
                });

                let client_id = &self.client_metadata.client_id;

                // Create JWT for client authentication
                let jwt = keyset
                    .create_jwt(
                        json!({
                            "alg": alg[0],
                            "kid": kid.as_ref().and_then(|k| k.first())
                        }),
                        json!({
                            "iss": client_id,
                            "sub": client_id,
                            "aud": self.server_metadata.issuer,
                            "jti": self.runtime.generate_nonce().await?,
                            "iat": chrono::Utc::now().timestamp()
                        }),
                    )
                    .await?;

                return Ok(ClientAuth {
                    headers: None,
                    payload: Some(json!({
                        "client_id": client_id,
                        "client_assertion_type": CLIENT_ASSERTION_TYPE_JWT_BEARER,
                        "client_assertion": jwt,
                    })),
                });
            } else if method == Some("private_key_jwt") {
                return Err(anyhow::anyhow!(
                    "No keyset available for private_key_jwt authentication"
                ));
            }
        }

        // Default to "none" authentication method
        if method == Some("none")
            || (method.is_none()
                && (method_supported.contains(&"none".to_string()) || method_supported.is_empty()))
        {
            return Ok(ClientAuth {
                headers: None,
                payload: Some(json!({
                    "client_id": self.client_metadata.client_id
                })),
            });
        }

        Err(anyhow::anyhow!(
            "Unsupported {} authentication method",
            endpoint
        ))
    }
}

/// Authentication information for client requests
#[derive(Debug)]
struct ClientAuth {
    headers: Option<HashMap<String, String>>,
    payload: Option<serde_json::Value>,
}

/// Extension trait for OAuthAuthorizationServerMetadata
trait OAuthServerMetadataExt {
    fn get_endpoint_url(&self, endpoint: &str) -> Result<String>;
    fn get_token_endpoint_auth_methods_supported(&self) -> Vec<String>;
}

impl OAuthServerMetadataExt for OAuthAuthorizationServerMetadata {
    fn get_endpoint_url(&self, endpoint: &str) -> Result<String> {
        let url = match endpoint {
            "token" => Some(&self.token_endpoint),
            "revocation" => self.revocation_endpoint.as_ref(),
            "pushed_authorization_request" => self.pushed_authorization_request_endpoint.as_ref(),
            _ => None,
        };

        url.map(|u| u.to_string())
            .ok_or_else(|| anyhow::anyhow!("No {} endpoint available", endpoint))
    }

    fn get_token_endpoint_auth_methods_supported(&self) -> Vec<String> {
        self.token_endpoint_auth_methods_supported
            .clone()
            .unwrap_or_default()
    }
}

/// Function to validate AtprotoTokenResponse schema
fn atproto_token_response_schema(json: Json) -> Result<AtprotoTokenResponse> {
    // In a real implementation, this would validate against a schema
    Ok(serde_json::from_value(json)?)
}

/// Function to validate OAuthParResponse schema
fn oauth_par_response_schema(json: Json) -> Result<OAuthParResponse> {
    // In a real implementation, this would validate against a schema
    Ok(serde_json::from_value(json)?)
}
