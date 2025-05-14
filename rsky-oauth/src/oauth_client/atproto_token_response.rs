/// Represents a space-separated string value (like OAuth scopes)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpaceSeparatedValue(pub String);

/// Utility functions for space-separated values
pub mod space_separated_values {
    /// Checks if the input string includes the specified value in its space-separated values
    pub fn includes_space_separated_value(input: &str, value: &str) -> bool {
        input.split_whitespace().any(|s| s == value)
    }

    /// Joins multiple values into a space-separated string
    pub fn join_space_separated_values(values: &[&str]) -> String {
        values.join(" ")
    }
}

use serde::{Deserialize, Serialize};

/// AtprotoScope represents a space-separated value containing 'atproto'
pub type AtprotoScope = SpaceSeparatedValue;

/// Checks if the input string includes 'atproto' in its space-separated values
pub fn is_atproto_scope(input: &str) -> bool {
    input.split_whitespace().any(|s| s == "atproto")
}

/// Validates that a scope includes the required 'atproto' value
pub fn validate_atproto_scope(scope: &str) -> Result<(), String> {
    if is_atproto_scope(scope) {
        Ok(())
    } else {
        Err("The \"atproto\" scope is required".to_string())
    }
}

/// Token response schema for AT Protocol OAuth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtprotoTokenResponse {
    pub access_token: String,
    pub token_type: String, // Must be "DPoP"
    pub sub: String,        // Must be a valid AT Protocol DID
    pub scope: String,      // Must include "atproto"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_in: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    // OpenID is not compatible with atproto identities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_token: Option<String>,
}

/// Validates an OAuthTokenResponse against the AT Protocol requirements
pub fn validate_atproto_token_response(response: &AtprotoTokenResponse) -> Result<(), String> {
    // Validate token_type is "DPoP"
    if response.token_type != "DPoP" {
        return Err("token_type must be 'DPoP'".to_string());
    }

    // Validate did
    validate_atproto_did(&response.sub)?;

    // Validate scope includes "atproto"
    validate_atproto_scope(&response.scope)?;

    // Validate id_token is not present (OpenID not compatible)
    if response.id_token.is_some() {
        return Err("id_token is not compatible with atproto identities".to_string());
    }

    Ok(())
}

/// Validates that a string is a valid AT Protocol DID
pub fn validate_atproto_did(did: &str) -> Result<(), String> {
    if !did.starts_with("did:") {
        return Err("Invalid DID format: must start with 'did:'".to_string());
    }
    // In a real implementation, this would have more thorough validation
    Ok(())
}

/// Parses and validates an OAuth token response into an AtprotoTokenResponse
pub fn atproto_token_response_schema(
    value: serde_json::Value,
) -> Result<AtprotoTokenResponse, String> {
    let response: AtprotoTokenResponse = serde_json::from_value(value)
        .map_err(|e| format!("Failed to parse token response: {}", e))?;

    validate_atproto_token_response(&response)?;

    Ok(response)
}

/// Implementation of SpaceSeparatedValue for the AT Protocol OAuth scope
impl SpaceSeparatedValue {
    /// Creates a new SpaceSeparatedValue
    pub fn new(value: String) -> Self {
        SpaceSeparatedValue(value)
    }

    /// Checks if this value contains the specified scope
    pub fn contains(&self, scope: &str) -> bool {
        self.0.split_whitespace().any(|s| s == scope)
    }

    /// Checks if this is a valid AT Protocol scope (contains "atproto")
    pub fn is_atproto_scope(&self) -> bool {
        self.contains("atproto")
    }
}

impl AsRef<str> for SpaceSeparatedValue {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<String> for SpaceSeparatedValue {
    fn from(value: String) -> Self {
        SpaceSeparatedValue(value)
    }
}

impl From<&str> for SpaceSeparatedValue {
    fn from(value: &str) -> Self {
        SpaceSeparatedValue(value.to_string())
    }
}

impl std::fmt::Display for SpaceSeparatedValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
