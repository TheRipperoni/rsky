use crate::account_manager::helpers::account::ActorAccount;
use crate::account_manager::AccountManager;
use crate::apis::ApiError;
use crate::{SharedIdResolver, APP_USER_AGENT};
use anyhow::{bail, Result};
use rocket::serde::json::Json;
use rocket::State;
use rsky_common::env::{env_list, env_str};
use rsky_lexicon::com::atproto::identity::ResolveHandleOutput;

#[tracing::instrument(skip_all)]
async fn try_resolve_from_app_view(handle: &String) -> Result<Option<String>> {
    match env_str("PDS_BSKY_APP_VIEW_URL") {
        None => Ok(None),
        Some(bsky_app_view_url) => {
            tracing::info!("Resolving from AppView");
            let client = reqwest::Client::builder()
                .user_agent(APP_USER_AGENT)
                .build()?;
            let params = Some(vec![("handle", handle)]);
            let res = client
                .get(format!(
                    "{bsky_app_view_url}/xrpc/com.atproto.identity.resolveHandle"
                ))
                .header("Connection", "Keep-Alive")
                .header("Keep-Alive", "timeout=5, max=1000")
                .query(&params)
                .send()
                .await;
            match res {
                Err(_) => {
                    tracing::error!("Failed to resolve handle from AppView");
                    Ok(None)
                }
                Ok(res) => match res.json::<ResolveHandleOutput>().await {
                    Err(_) => {
                        tracing::error!("No handle found from AppView");
                        Ok(None)
                    }
                    Ok(data) => Ok(Some(data.did)),
                },
            }
        }
    }
}

async fn inner_resolve_handle(
    handle: String,
    id_resolver: &State<SharedIdResolver>,
    account_manager: AccountManager,
) -> Result<ResolveHandleOutput, ApiError> {
    // @TODO: Implement normalizeAndEnsureValidHandle()
    let mut did: Option<String> = None;
    let user: Option<ActorAccount> = match account_manager.get_account(&handle, None).await {
        Ok(user) => user,
        Err(error) => {
            tracing::error!("Error getting account from account_manager: {error}");
            return Err(ApiError::RuntimeError);
        }
    };

    match user {
        Some(user) => did = Some(user.did),
        None => {
            let supported_handle = env_list("PDS_SERVICE_HANDLE_DOMAINS")
                .iter()
                .any(|host| handle.ends_with(host.as_str()) || handle == host[1..]);
            // this should be in our DB & we couldn't find it, so fail
            if supported_handle {
                return Err(ApiError::HandleNotFound(
                    "unable to resolve handle".to_string(),
                ));
            }
        }
    }

    // this is not someone on our server, but we help with resolving anyway
    // @TODO: Weird error about Tokio received when this fails that leads to panic
    if did.is_none() && env_str("PDS_BSKY_APP_VIEW_URL").is_some() {
        did = match try_resolve_from_app_view(&handle).await {
            Ok(did) => did,
            Err(error) => {
                tracing::error!("Error getting account from account_manager: {error}");
                return Err(ApiError::RuntimeError);
            }
        };
    }

    if did.is_none() {
        let mut lock = id_resolver.id_resolver.write().await;
        did = match lock.handle.resolve(&handle).await {
            Ok(did) => did,
            Err(error) => {
                tracing::error!("Error getting account from account_manager: {error}");
                return Err(ApiError::RuntimeError);
            }
        };
        drop(lock);
    }

    match did {
        None => Err(ApiError::HandleNotFound(
            "unable to resolve handle".to_string(),
        )),
        Some(did) => Ok(ResolveHandleOutput { did }),
    }
}

#[tracing::instrument(skip(id_resolver, account_manager))]
#[rocket::get("/xrpc/com.atproto.identity.resolveHandle?<handle>")]
pub async fn resolve_handle(
    handle: String,
    id_resolver: &State<SharedIdResolver>,
    account_manager: AccountManager,
) -> Result<Json<ResolveHandleOutput>, ApiError> {
    match inner_resolve_handle(handle, id_resolver, account_manager).await {
        Ok(res) => Ok(Json(res)),
        Err(error) => {
            tracing::error!("@LOG: ERROR: {error}");
            Err(error)
        }
    }
}
