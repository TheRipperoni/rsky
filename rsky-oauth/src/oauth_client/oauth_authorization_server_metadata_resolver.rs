use crate::cached_getter::{CachedGetter, GetCachedOptions};
use crate::oauth_client::oauth_protected_resource_metadata_resolver::ProtectedResourceMetadataCache;
use crate::oauth_types::{OAuthAuthorizationServerMetadata, OAuthProtectedResourceMetadata};
use crate::simple_store::SimpleStore;
use url::Url;

pub type AuthorizationServerMetadataCache =
    Box<dyn SimpleStore<String, OAuthAuthorizationServerMetadata, Error = ()>>;

pub struct OAuthProtectedResourceMetadataResolverConfig {
    pub allow_http_issuer: bool,
}

/**
 * @see {@link https://datatracker.ietf.org/doc/html/draft-ietf-oauth-resource-metadata-05}
 */
pub struct OAuthAuthorizationServerMetadataResolver {
    allow_http_issuer: bool,
}

impl OAuthAuthorizationServerMetadataResolver {
    pub fn new(
        allow_http_issuer: bool,
        cache: AuthorizationServerMetadataCache,
        config: Option<OAuthProtectedResourceMetadataResolverConfig>,
    ) -> Self {
        Self { allow_http_issuer }
    }

    pub async fn get(&self, resource: Url, options: Option<GetCachedOptions>) {
        if resource.scheme() != "http" && resource.scheme() != "https" {
            panic!("Invalid protected resource metadata URL protocol: ");
        }

        if resource.scheme() == "http" && !self.allow_http_resource {
            panic!("Unsecure resource metadata URL");
        }
    }

    async fn fetch_metadata(&self) {
        unimplemented!()
    }
}
