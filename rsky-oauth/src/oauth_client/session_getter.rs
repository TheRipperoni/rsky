use crate::cached_getter::{CachedGetter, GetCachedOptions};
use crate::jwk::Key;
use crate::oauth_client::oauth_server_agent::TokenSet;
use crate::oauth_client::oauth_server_factory::OAuthServerFactory;
use crate::simple_store::SimpleStore;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::broadcast;

/// Represents a session with an OAuth server
#[derive(Clone, Debug)]
pub struct Session {
    pub dpop_key: dyn Key,
    pub token_set: TokenSet,
}

/// Type alias for a store that stores sessions
pub type SessionStore = dyn SimpleStore<String, Session, Error = ()>;

/// Enum for session events
#[derive(Clone, Debug)]
pub enum SessionEventMap {
    Updated {
        sub: String,
        dpop_key: Key,
        token_set: TokenSet,
    },
    Deleted {
        sub: String,
        cause: Arc<anyhow::Error>,
    },
}

/// SessionGetter wraps a session store in a CachedGetter to ensure
/// at most one fresh call is ever being made and to handle caching logic
pub struct SessionGetter {
    cached_getter: CachedGetter<AtprotoDid, Session>,
    event_sender: Sender<SessionEvent>,
    server_factory: Arc<OAuthServerFactory>,
    runtime: Arc<Runtime>,
}

impl SessionGetter {
    pub fn new(
        session_store: SessionStore,
        server_factory: OAuthServerFactory,
        runtime: Runtime,
    ) -> Self {
        let (event_sender, _) = broadcast::channel(100);
        let event_sender_clone = event_sender.clone();
        let server_factory = Arc::new(server_factory);
        let runtime = Arc::new(runtime);

        let server_factory_clone = server_factory.clone();
        let runtime_clone = runtime.clone();

        let cached_getter = CachedGetter::new(
            // Getter function
            move |sub: AtprotoDid,
                  options: Option<GetCachedOptions>,
                  stored_session: Option<Session>| {
                let server_factory = server_factory_clone.clone();
                let runtime = runtime_clone.clone();
                let event_sender = event_sender.clone();

                Box::pin(async move {
                    // There needs to be a previous session to be able to refresh
                    let stored_session = match stored_session {
                        Some(session) => session,
                        None => {
                            // Session not in store, dispatch deleted event
                            let msg = "The session was deleted by another process";
                            let cause = Arc::new(TokenRefreshError::new(&sub, msg).into());
                            let _ = event_sender.send(SessionEvent::Deleted {
                                sub: sub.clone(),
                                cause: cause.clone(),
                            });
                            return Err(anyhow::anyhow!(cause));
                        }
                    };

                    let Session {
                        dpop_key,
                        token_set,
                    } = stored_session;

                    if sub != token_set.sub {
                        // Fool-proofing against invalid session storage
                        return Err(anyhow::anyhow!(TokenRefreshError::new(
                            &sub,
                            "Stored session sub mismatch",
                        )));
                    }

                    if token_set.refresh_token.is_none() {
                        return Err(anyhow::anyhow!(TokenRefreshError::new(
                            &sub,
                            "No refresh token available",
                        )));
                    }

                    // Check for abort signal before proceeding
                    if let Some(opts) = &options {
                        if let Some(signal) = &opts.signal {
                            if signal.is_aborted() {
                                return Err(anyhow::anyhow!("Operation aborted"));
                            }
                        }
                    }

                    let server = server_factory
                        .from_issuer(&token_set.iss, dpop_key.clone())
                        .await?;

                    // Try to refresh the token
                    match server.refresh(token_set.clone()).await {
                        Ok(new_token_set) => {
                            if sub != new_token_set.sub {
                                // The server returned another sub
                                return Err(anyhow::anyhow!(TokenRefreshError::new(
                                    &sub,
                                    "Token set sub mismatch",
                                )));
                            }

                            Ok(Session {
                                dpop_key,
                                token_set: new_token_set,
                            })
                        }
                        Err(err) => {
                            // Handle refresh token invalid error
                            if let Some(oauth_err) = err.downcast_ref::<OAuthResponseError>() {
                                if oauth_err.status == 400 && oauth_err.error == "invalid_grant" {
                                    // In case there is no lock implementation, wait for concurrent refreshes
                                    if !runtime.has_implementation_lock() {
                                        tokio::time::sleep(Duration::from_millis(1000)).await;

                                        // Check if session was updated by another process
                                        match session_store.get(&sub).await {
                                            Ok(None) => {
                                                // Session was deleted by another process
                                                let msg =
                                                    "The session was deleted by another process";
                                                return Err(anyhow::anyhow!(
                                                    TokenRefreshError::new(&sub, msg,)
                                                ));
                                            }
                                            Ok(Some(stored)) => {
                                                // Check if tokens are different (indicating a concurrent refresh)
                                                if stored.token_set.access_token
                                                    != token_set.access_token
                                                    || stored.token_set.refresh_token
                                                        != token_set.refresh_token
                                                {
                                                    // A concurrent refresh occurred, use that session
                                                    return Ok(stored);
                                                }
                                            }
                                            Err(store_err) => return Err(store_err),
                                        }
                                    }

                                    // Token is no longer valid
                                    let msg = oauth_err
                                        .error_description
                                        .clone()
                                        .unwrap_or_else(|| "The session was revoked".to_string());

                                    return Err(anyhow::anyhow!(TokenRefreshError::new(
                                        &sub, &msg
                                    )));
                                }
                            }

                            // Other errors just propagate
                            Err(err)
                        }
                    }
                })
            },
            session_store,
            // Options
            {
                let runtime = runtime.clone();
                let server_factory = server_factory.clone();

                move |key: &AtprotoDid, session: &Session| {
                    let is_stale = match &session.token_set.expires_at {
                        Some(expires_at) => {
                            let timestamp = expires_at.timestamp() * 1000; // Convert to milliseconds
                            let now = chrono::Utc::now().timestamp() * 1000;

                            // Add leeway (10 seconds) and randomness (0-30 seconds)
                            let leeway = 10_000;
                            let randomness = (rand::random::<f64>() * 30_000.0) as i64;

                            timestamp < now + leeway + randomness
                        }
                        None => false,
                    };

                    is_stale
                }
            },
            // On store error callback
            {
                let server_factory = server_factory.clone();

                move |err, sub, session| {
                    let server_factory = server_factory.clone();
                    let token_set = session.token_set.clone();
                    let dpop_key = session.dpop_key.clone();

                    Box::pin(async move {
                        // If token data can't be stored, revoke it
                        if let Ok(server) =
                            server_factory.from_issuer(&token_set.iss, dpop_key).await
                        {
                            let token_to_revoke = token_set
                                .refresh_token
                                .unwrap_or_else(|| token_set.access_token.clone());

                            let _ = server.revoke(&token_to_revoke).await;
                        }

                        Err(err)
                    })
                }
            },
            // Delete on error predicate
            move |err| {
                err.downcast_ref::<TokenRefreshError>().is_some()
                    || err.downcast_ref::<TokenRevokedError>().is_some()
                    || err.downcast_ref::<TokenInvalidError>().is_some()
            },
        );

        Self {
            cached_getter,
            event_sender: event_sender_clone,
            server_factory,
            runtime,
        }
    }

    /// Subscribe to session events
    pub fn subscribe(&self) -> Receiver<SessionEvent> {
        self.event_sender.subscribe()
    }

    /// Set a session in the store
    pub async fn set_stored(&self, sub: String, session: Session) -> Result<()> {
        // Prevent tampering with the stored value
        if sub != session.token_set.sub {
            return Err(anyhow::anyhow!("Token set does not match the expected sub"));
        }

        self.cached_getter.set_stored(&sub, session.clone()).await?;

        // Dispatch updated event
        let _ = self.event_sender.send(SessionEvent::Updated {
            sub,
            dpop_key: session.dpop_key,
            token_set: session.token_set,
        });

        Ok(())
    }

    /// Delete a session from the store
    pub async fn del_stored(
        &self,
        sub: &AtprotoDid,
        cause: Box<dyn std::error::Error + Send + Sync>,
    ) -> Result<()> {
        self.cached_getter.del_stored(sub).await?;

        // Dispatch deleted event
        let _ = self.event_sender.send(SessionEvent::Deleted {
            sub: sub.clone(),
            cause: Arc::new(anyhow::anyhow!(cause)),
        });

        Ok(())
    }

    /// Get a session, with optional refresh behavior
    pub async fn get_session(&self, sub: &AtprotoDid, refresh: Option<bool>) -> Result<Session> {
        self.get(
            sub,
            GetCachedOptions {
                no_cache: refresh.unwrap_or(false),
                allow_stale: !refresh.unwrap_or(true),
                signal: None,
            },
        )
        .await
    }

    /// Get a session with more options
    pub async fn get(&self, sub: &AtprotoDid, options: GetCachedOptions) -> Result<Session> {
        let lock_key = format!("@atproto-oauth-client-{}", sub);

        let session = self
            .runtime
            .using_lock(&lock_key, async {
                // Set up timeout signal (30 seconds)
                let timeout_signal = timeout_signal(Duration::from_secs(30));

                // Combine with optional user signal
                let abort_controller =
                    combine_signals(vec![options.signal.clone(), Some(timeout_signal)]);

                // Get with combined signal
                let mut new_options = options.clone();
                new_options.signal = Some(abort_controller.signal.clone());

                self.cached_getter.get(sub, new_options).await
            })
            .await?;

        // Fool-proofing
        if sub != &session.token_set.sub {
            return Err(anyhow::anyhow!("Token set does not match the expected sub"));
        }

        Ok(session)
    }

    /// Get a stored session directly without refreshing
    pub async fn get_stored(&self, sub: &AtprotoDid) -> Result<Option<Session>> {
        self.cached_getter.get_stored(sub).await
    }
}
