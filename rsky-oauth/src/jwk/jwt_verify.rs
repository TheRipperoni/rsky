use crate::jwk::{JwtHeader, JwtPayload};
use crate::oauth_types::OAuthIssuerIdentifier;
use biscuit::jwk::JWK;
use biscuit::Empty;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct VerifyOptions {
    pub audience: Option<String>,
    /** in seconds */
    pub clock_tolerance: Option<i64>,
    pub issuer: Option<OAuthIssuerIdentifier>,
    /** in seconds */
    pub max_token_age: Option<i64>,
    pub subject: Option<String>,
    pub typ: Option<String>,
    pub current_date: Option<i64>,
    pub required_claims: Vec<String>,
}

#[derive(Serialize, Eq, PartialEq, Deserialize, Debug)]
pub enum DecodeRequestObjectResult {
    UnsecuredResult(UnsecuredResult),
    SecuredResult(VerifyResult),
}

impl DecodeRequestObjectResult {
    pub fn payload(&self) -> &JwtPayload {
        match self {
            DecodeRequestObjectResult::UnsecuredResult(result) => &result.payload,
            DecodeRequestObjectResult::SecuredResult(result) => &result.payload,
        }
    }

    pub fn new() -> Self {
        unimplemented!()
    }
}

#[derive(Serialize, Eq, PartialEq, Deserialize, Debug)]
pub struct VerifyResult {
    pub payload: JwtPayload,
    pub protected_header: JwtHeader,
    pub key: JWK<Empty>,
}

#[derive(Serialize, Eq, PartialEq, Deserialize, Debug)]
pub struct UnsecuredResult {
    pub payload: JwtPayload,
    pub header: JwtHeader,
}
