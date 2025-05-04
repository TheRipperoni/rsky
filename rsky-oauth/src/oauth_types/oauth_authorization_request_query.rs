use serde::{Deserialize, Serialize};
use std::fmt;

use crate::oauth_types::{
    OAuthAuthorizationRequestJar, OAuthAuthorizationRequestParameters, OAuthAuthorizationRequestUri,
};

/// An OAuth authorization request query.
///
/// This represents the different ways an authorization request can be made:
/// - Direct parameters in the query
/// - JWT-based request (JAR)
/// - Request URI reference
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OAuthAuthorizationRequestQuery {
    /// Standard authorization request parameters
    Parameters(OAuthAuthorizationRequestParameters),
    /// JWT-based request parameters
    Jar(OAuthAuthorizationRequestJar),
    /// Reference to pushed request parameters
    Uri(OAuthAuthorizationRequestUri),
}

impl OAuthAuthorizationRequestQuery {
    /// Create a new query from authorization request parameters.
    pub fn from_parameters(params: OAuthAuthorizationRequestParameters) -> Self {
        Self::Parameters(params)
    }

    /// Create a new query from a JWT-based authorization request.
    pub fn from_jar(jar: OAuthAuthorizationRequestJar) -> Self {
        Self::Jar(jar)
    }

    /// Create a new query from a request URI reference.
    pub fn from_uri(uri: OAuthAuthorizationRequestUri) -> Self {
        Self::Uri(uri)
    }

    /// Returns true if this is a parameters-based request.
    pub fn is_parameters(&self) -> bool {
        matches!(self, Self::Parameters(_))
    }

    /// Returns true if this is a JAR-based request.
    pub fn is_jar(&self) -> bool {
        matches!(self, Self::Jar(_))
    }

    /// Returns true if this is a URI-based request.
    pub fn is_uri(&self) -> bool {
        matches!(self, Self::Uri(_))
    }

    /// Get the inner parameters if this is a parameters-based request.
    pub fn as_parameters(&self) -> Option<&OAuthAuthorizationRequestParameters> {
        match self {
            Self::Parameters(params) => Some(params),
            _ => None,
        }
    }

    /// Get the inner JAR if this is a JAR-based request.
    pub fn as_jar(&self) -> Option<&OAuthAuthorizationRequestJar> {
        match self {
            Self::Jar(jar) => Some(jar),
            _ => None,
        }
    }

    /// Get the inner URI if this is a URI-based request.
    pub fn as_uri(&self) -> Option<&OAuthAuthorizationRequestUri> {
        match self {
            Self::Uri(uri) => Some(uri),
            _ => None,
        }
    }
}

impl fmt::Display for OAuthAuthorizationRequestQuery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parameters(params) => write!(f, "Parameters({})", params),
            Self::Jar(jar) => write!(f, "JAR({})", jar),
            Self::Uri(uri) => write!(f, "URI({})", uri),
        }
    }
}
