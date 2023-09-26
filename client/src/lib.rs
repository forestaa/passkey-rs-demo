mod utils;

use base64urlsafedata::Base64UrlSafeData;
use js_sys::{ArrayBuffer, Uint8Array};
use once_cell::sync::Lazy;
use passkey::{
    authenticator::{Authenticator, UserValidationMethod},
    client::Client,
    types::{ctap2::Aaguid, Passkey},
};
use std::sync::Mutex;
use url::Url;
use wasm_bindgen::prelude::*;
use web_sys::PublicKeyCredential;

struct MyUserValidationMethod;

#[async_trait::async_trait]
impl UserValidationMethod for MyUserValidationMethod {
    async fn check_user_presence(&self) -> bool {
        true
    }

    async fn check_user_verification(&self) -> bool {
        true
    }

    fn is_verification_enabled(&self) -> Option<bool> {
        Some(true)
    }

    fn is_presence_enabled(&self) -> bool {
        true
    }
}

static CLIENT: Lazy<
    Mutex<Client<Option<Passkey>, MyUserValidationMethod, public_suffix::PublicSuffixList>>,
> = Lazy::new(|| {
    let my_aaguid = Aaguid::new_empty();
    let user_validation_method = MyUserValidationMethod {};
    let store: Option<Passkey> = None;
    let my_authenticator = Authenticator::new(my_aaguid, store, user_validation_method);
    Mutex::new(Client::new(my_authenticator))
});

#[wasm_bindgen]
pub async fn register(url: String, credential_creation_options: JsValue) -> PublicKeyCredential {
    utils::set_panic_hook();
    let url = Url::parse(&url).unwrap();
    let credential_creation_options =
        serde_wasm_bindgen::from_value(credential_creation_options).unwrap();
    let public_key = CLIENT
        .lock()
        .unwrap()
        .register(&url, credential_creation_options)
        .await
        .unwrap();
    PublicKeyCredential::from(serde_wasm_bindgen::to_value(&public_key).unwrap())
}

#[wasm_bindgen]
pub fn encode_base64_url(uarray: ArrayBuffer) -> String {
    Base64UrlSafeData::from(Uint8Array::new(&uarray).to_vec()).to_string()
}
