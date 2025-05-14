use crate::cached_getter::{CachedGetter, GetCachedOptions};
use crate::oauth_types::OAuthProtectedResourceMetadata;
use crate::simple_store::SimpleStore;
use url::Url;

pub type ProtectedResourceMetadataCache =
    Box<dyn SimpleStore<String, OAuthProtectedResourceMetadata, Error = ()>>;

pub struct OAuthProtectedResourceMetadataResolverConfig {
    pub allow_http_resource: bool,
}

/**
 * @see {@link https://datatracker.ietf.org/doc/html/draft-ietf-oauth-resource-metadata-05}
 */
pub struct OAuthProtectedResourceMetadataResolver {
    allow_http_resource: bool,
}

impl OAuthProtectedResourceMetadataResolver {
    pub fn new(
        allow_http_resource: bool,
        cache: ProtectedResourceMetadataCache,
        config: Option<OAuthProtectedResourceMetadataResolverConfig>,
    ) -> Self {
        Self {
            allow_http_resource,
        }
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
