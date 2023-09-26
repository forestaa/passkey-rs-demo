use std::{collections::HashMap, sync::RwLock};
use webauthn_rs::prelude::{CredentialID, Passkey, Uuid};

#[derive(Debug, Clone)]
pub(crate) struct User {
    pub(crate) id: Uuid,
    pub(crate) email: String,
    passkey: Option<Passkey>,
}

impl User {
    fn new(id: Uuid, email: String, passkey: Option<Passkey>) -> Self {
        Self { id, email, passkey }
    }

    pub(crate) fn create(email: String) -> Self {
        Self::new(Uuid::new_v4(), email, None)
    }

    pub(crate) fn get_passkey(&self) -> Option<&Passkey> {
        self.passkey.as_ref()
    }

    pub(crate) fn update_passkey(&mut self, passkey: Passkey) {
        self.passkey.replace(passkey);
    }
}

pub(crate) struct UserRepository(RwLock<UserRepositoryBody>);

struct UserRepositoryBody {
    users: HashMap<Uuid, User>,
    passkey_index: HashMap<CredentialID, Uuid>,
}

impl UserRepository {
    pub(crate) fn new() -> Self {
        Self(RwLock::new(UserRepositoryBody {
            users: HashMap::new(),
            passkey_index: HashMap::new(),
        }))
    }

    pub(crate) fn save_user(&self, user: User) {
        let mut wlock = self.0.write().unwrap();
        if let Some(ref passkey) = user.passkey {
            wlock.passkey_index.remove(passkey.cred_id());
            wlock
                .passkey_index
                .insert(passkey.cred_id().clone(), user.id);
        }
        wlock.users.insert(user.id, user);
    }

    pub(crate) fn fetch_user(&self, user_id: &Uuid) -> Option<User> {
        self.0.read().unwrap().users.get(user_id).cloned()
    }

    pub(crate) fn fetch_user_by_passkey(&self, credential_id: &CredentialID) -> Option<User> {
        let rlock = self.0.read().unwrap();
        rlock
            .passkey_index
            .get(credential_id)
            .and_then(|user_id| rlock.users.get(user_id))
            .cloned()
    }
}
