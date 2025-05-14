use crate::cached_getter::GetCachedOptions;
use crate::jwk::{Key, Keyset};
use crate::oauth_client::oauth_server_agent::{DpopNonceCache, OAuthServerAgent};
use crate::oauth_client::oauth_session::OAuthAuthorizationServerMetadata;
use crate::oauth_client::types::ClientMetadata;
use tokio::runtime::Runtime;

pub struct OAuthServerFactory {
    client_metadata: ClientMetadata,
    runtime: Runtime,
    resolver: OAuthResolver,
    fetch: Fetch,
    keyset: Option<Keyset>,
    dpop_nonce_cache: DpopNonceCache,
}

impl OAuthServerFactory {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn from_issuer(
        issuer: &str,
        dpop_key: Key,
        options: Option<GetCachedOptions>,
    ) -> OAuthServerAgent {
        unimplemented!()
    }

    pub async fn from_metadata(
        server_metadata: OAuthAuthorizationServerMetadata,
        dpop_key: Key,
    ) -> OAuthServerAgent {
        unimplemented!()
    }
}
