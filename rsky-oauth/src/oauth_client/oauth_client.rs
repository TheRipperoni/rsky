use crate::jwk::Keyset;
use crate::oauth_client::oauth_authorization_server_metadata_resolver::AuthorizationServerMetadataCache;
use crate::oauth_client::oauth_protected_resource_metadata_resolver::ProtectedResourceMetadataCache;
use crate::oauth_client::oauth_resolver::OAuthResolver;
use crate::oauth_client::oauth_server_agent::{DpopNonceCache, OAuthServerAgent};
use crate::oauth_client::oauth_server_factory::OAuthServerFactory;
use crate::oauth_client::oauth_session::OAuthSession;
use crate::oauth_client::session_getter::{SessionEventMap, SessionGetter, SessionStore};
use crate::oauth_client::state_store::StateStore;
use crate::oauth_provider::oidc::sub::Sub;
use crate::oauth_types::{OAuthClientIdDiscoverable, OAuthClientMetadata, OAuthResponseMode};
use rsky_identity::handle::HandleResolver;
use rsky_identity::types::DidCache;
use rsky_identity::IdResolver;
use tokio::runtime::Runtime;
use url::Url;

pub struct OAuthClientOptions {
    // Config
    pub oauth_response_mode: OAuthResponseMode,
    client_metadata: OAuthClientMetadata,
    pub keyset: Option<Keyset>,
    /**
     * Determines if the client will allow communicating with the OAuth Servers
     * (Authorization & Resource), or to retrieve "did:web" documents, over
     * unsafe HTTP connections. It is recommended to set this to `true` only for
     * development purposes.
     *
     * @note This does not affect the identity resolution mechanism, which will
     * allow HTTP connections to the PLC Directory (if the provided directory url
     * is "http:" based).
     * @default false
     * @see {@link OAuthProtectedResourceMetadataResolver.allowHttpResource}
     * @see {@link OAuthAuthorizationServerMetadataResolver.allowHttpIssuer}
     * @see {@link DidResolverCommonOptions.allowHttp}
     */
    pub allow_http: bool,

    // Stores
    pub state_store: StateStore,
    pub session_store: SessionStore,
    pub did_cache: Option<DidCache>,
    pub handle_cache: Option<HandleCache>,
    pub authorization_server_metadata_cache: Option<AuthorizationServerMetadataCache>,
    pub protected_resource_metadata_cache: Option<ProtectedResourceMetadataCache>,
    pub dpop_nonce_cache: Option<DpopNonceCache>,

    // Services
    pub handle_resolver: HandleResolver,
    pub plc_directory_url: Option<Url>,
    pub runtime_implementation: RuntimeImplementation,
}

pub type OAuthClientEventMap = SessionEventMap;

pub struct OAuthClientFetchMetadataOptions {
    client_id: OAuthClientIdDiscoverable,
}

pub struct OAuthClient {
    client_metadata: OAuthClientMetadata,
    oauth_response_mode: OAuthResponseMode,
    keyset: Option<Keyset>,

    runtime: Runtime,
    oauth_resolver: OAuthResolver,
    server_factory: OAuthServerFactory,

    session_getter: SessionGetter,
    state_store: StateStore,
}

impl OAuthClient {
    pub async fn fetch_metadata() {
        unimplemented!()
    }

    pub fn identity_resolver(&self) -> &IdResolver {
        &self.oauth_resolver.identity_resolver
    }
    pub fn did_resolver(&self) {
        unimplemented!()
    }

    pub fn handle_resolver(&self) {
        unimplemented!()
    }

    pub fn jwks(&self) {
        unimplemented!()
    }

    pub async fn authorize(&self, input: &str) {
        unimplemented!()
    }

    pub async fn abort_request(&self, authorize_url: Url) {
        unimplemented!()
    }
    /**
     * Load a stored session. This will refresh the token only if needed (about to
     * expire) by default.
     *
     * @param refresh See {@link SessionGetter.getSession}
     */
    pub async fn restore(&self, sub: Sub, refresh: bool) -> OAuthSession {
        unimplemented!()
    }

    pub async fn revoke(&self, sub: Sub) {
        unimplemented!()
    }

    fn create_session(&self, server: OAuthServerAgent, sub: Sub) -> OAuthSession {
        OAuthSession::new(server, sub, self.session_getter.clone())
    }
}
