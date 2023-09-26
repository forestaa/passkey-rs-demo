use actix_session::storage::{LoadError, SaveError, SessionKey, SessionStore, UpdateError};
use actix_web::cookie::time::{Duration, Instant};
use anyhow::anyhow;
use once_cell::sync::Lazy;
use rand::distributions::{Alphanumeric, DistString};
use std::{collections::HashMap, ops::Add, sync::RwLock};

type SessionState = HashMap<String, String>;

struct Expirable<T> {
    value: T,
    expire: Instant,
}

impl<T> Expirable<T> {
    fn get(&self) -> Option<&T> {
        if self.expire < Instant::now() {
            None
        } else {
            Some(&self.value)
        }
    }
}

struct InMemorySessionStoreState(RwLock<HashMap<String, Expirable<SessionState>>>);

impl InMemorySessionStoreState {
    fn new() -> InMemorySessionStoreState {
        InMemorySessionStoreState(RwLock::new(HashMap::new()))
    }

    fn get(&self, session_key: &SessionKey) -> Option<SessionState> {
        self.0
            .read()
            .unwrap()
            .get(session_key.as_ref())
            .and_then(|s| s.get())
            .cloned()
    }

    fn insert(&self, session_key: &SessionKey, session_state: SessionState, ttl: Duration) {
        let expire = Instant::now().add(ttl);
        let session_key = session_key.as_ref().to_string();
        self.0.write().unwrap().insert(
            session_key,
            Expirable {
                value: session_state,
                expire: expire,
            },
        );
    }

    fn delete(&self, session_key: &SessionKey) {
        self.0
            .write()
            .unwrap()
            .remove(&session_key.as_ref().to_string());
    }
}

static INMEMORY_SESSION_STORE_STATE: Lazy<InMemorySessionStoreState> =
    Lazy::new(|| InMemorySessionStoreState::new());
pub(crate) struct InMemorySessionStore;

#[async_trait::async_trait(?Send)]
impl SessionStore for InMemorySessionStore {
    async fn load(&self, session_key: &SessionKey) -> Result<Option<SessionState>, LoadError> {
        Ok(INMEMORY_SESSION_STORE_STATE.get(session_key))
    }

    async fn save(
        &self,
        session_state: SessionState,
        ttl: &Duration,
    ) -> Result<SessionKey, SaveError> {
        let session_key =
            SessionKey::try_from(Alphanumeric.sample_string(&mut rand::thread_rng(), 512))
                .map_err(|_| SaveError::Serialization(anyhow!("Invalid Session Key")))?;
        INMEMORY_SESSION_STORE_STATE.insert(&session_key, session_state, ttl.clone());

        Ok(session_key)
    }

    async fn update(
        &self,
        session_key: SessionKey,
        session_state: SessionState,
        ttl: &Duration,
    ) -> Result<SessionKey, UpdateError> {
        INMEMORY_SESSION_STORE_STATE.insert(&session_key, session_state, ttl.clone());
        Ok(session_key)
    }

    async fn update_ttl(
        &self,
        session_key: &SessionKey,
        ttl: &Duration,
    ) -> Result<(), anyhow::Error> {
        let session_state = INMEMORY_SESSION_STORE_STATE
            .get(session_key)
            .ok_or(anyhow!("not found"))?;
        INMEMORY_SESSION_STORE_STATE.insert(session_key, session_state, ttl.clone());
        Ok(())
    }

    async fn delete(&self, session_key: &SessionKey) -> Result<(), anyhow::Error> {
        INMEMORY_SESSION_STORE_STATE.delete(session_key);
        Ok(())
    }
}
