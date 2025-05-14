use crate::cached_getter::GetCachedOptions;
use crate::oauth_client::oauth_authorization_server_metadata_resolver::OAuthAuthorizationServerMetadataResolver;
use crate::oauth_client::oauth_protected_resource_metadata_resolver::OAuthProtectedResourceMetadataResolver;
use rsky_identity::IdResolver;

pub struct OAuthResolver {
    pub identity_resolver: IdResolver,
    pub protected_resource_metadata_resolver: OAuthProtectedResourceMetadataResolver,
    pub authorization_server_metadata: OAuthAuthorizationServerMetadataResolver,
}

impl OAuthResolver {
    pub fn new(
        identity_resolver: IdResolver,
        protected_resource_metadata_resolver: OAuthProtectedResourceMetadataResolver,
        authorization_server_metadata: OAuthAuthorizationServerMetadataResolver,
    ) -> Self {
        Self {
            identity_resolver,
            protected_resource_metadata_resolver,
            authorization_server_metadata,
        }
    }

    /**
     * @param input - A handle, DID, PDS URL or Entryway URL
     */
    pub async fn resolve(&self, input: &str, options: Option<ResolveOAuthOptions>) {
        unimplemented!()
    }

    /**
     * @note this method can be used to verify if a particular uri supports OAuth
     * based sign-in (for compatibility with legacy implementation).
     */
    pub async fn resolve_from_service(&self, input: &str, options: Option<ResolveOAuthOptions>) {
        unimplemented!()
    }

    pub async fn resolve_from_identity(&self, input: &str, options: Option<ResolveOAuthOptions>) {
        unimplemented!()
    }

    pub async fn resolve_identity(&self, input: &str, options: Option<ResolveIdentityOptions>) {
        unimplemented!()
    }

    pub async fn get_authorization_server_metadata(&self) {
        unimplemented!()
    }

    pub async fn get_resource_server_metadata(
        &self,
        pds_url: &str,
        options: Option<GetCachedOptions>,
    ) {
        unimplemented!()
    }
}
