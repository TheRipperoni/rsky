use crate::jwk::Keyset;
use crate::oauth_client::types::ClientMetadata;
use crate::oauth_types::{
    assert_oauth_discoverable_client_id, assert_oauth_loopback_client_id, OAuthClientMetadata,
};

const TOKEN_ENDPOINT_AUTH_METHOD: &str = "token_endpoint_auth_method";
const TOKEN_ENDPOINT_AUTH_SIGNING_ALG: &str = "token_endpoint_auth_signing_alg";

pub fn validate_client_metadata(
    input: OAuthClientMetadata,
    keyset: Option<Keyset>,
) -> ClientMetadata {
    if let Some(jwks) = &input.jwks {
        if keyset.is_some() {
            panic!("Keyset must not be provided when jwks is provided")
        }
        for key in jwks.keys {
            if let Some(kid) = key.common.key_id {
            } else {
                panic!("Key must have a \"kid\" property")
            }
        }
    }

    // Allow to pass a keyset and omit the jwks/jwks_uri properties
    if input.jwks.is_none() && input.jwks_uri.is_none() && keyset.is_some() {
        unimplemented!()
    }

    let metadata: ClientMetadata;

    // Validate client ID
    if metadata.client_id.starts_with("http") {
        assert_oauth_loopback_client_id(metadata.client_id.as_str()).unwrap();
    } else {
        assert_oauth_discoverable_client_id(metadata.client_id.as_str()).unwrap();
    }
}
