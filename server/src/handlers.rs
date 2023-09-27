use crate::user::{User, UserRepository};
use actix_identity::Identity;
use actix_session::Session;
use actix_web::{
    post,
    web::{Data, Json},
    HttpMessage, HttpRequest, HttpResponse, Responder,
};
use webauthn_rs::{
    prelude::{
        CreationChallengeResponse, Passkey, PasskeyRegistration, PublicKeyCredential,
        RegisterPublicKeyCredential, Url, Uuid,
    },
    Webauthn, WebauthnBuilder,
};

pub(crate) struct Application {
    user_repository: UserRepository,
    webauthn: Webauthn,
}

impl Application {
    pub fn new() -> Self {
        Self {
            user_repository: UserRepository::new(),
            webauthn: {
                let rp_id = "passkey-demo.localhost";
                let rp_origin = Url::parse("https://passkey-demo.localhost:8081").unwrap();
                WebauthnBuilder::new(rp_id, &rp_origin)
                    .unwrap()
                    .rp_name("my_rp")
                    .build()
                    .unwrap()
            },
        }
    }
}

#[post("/users/register")]
pub(crate) async fn register_user(
    application: Data<Application>,
    request: HttpRequest,
    email: String,
) -> impl Responder {
    let user = User::create(email);
    let user_id = user.id.to_string();
    application.user_repository.save_user(user);

    Identity::login(&request.extensions(), user_id.clone()).unwrap();

    HttpResponse::Created()
}

#[post("/passkey/register/start")]
pub(crate) async fn start_passkey_registration(
    session: Session,
    identity: Identity,
    application: Data<Application>,
) -> Json<CreationChallengeResponse> {
    let user_id = Uuid::parse_str(&identity.id().unwrap()).unwrap();
    let user = application.user_repository.fetch_user(&user_id).unwrap();

    let exclude_credentials = user
        .get_passkey()
        .map(|passkey| vec![passkey.cred_id().clone()]);
    let (ccr, state) = application
        .webauthn
        .start_passkey_registration(user.id, &user.email, &user.email, exclude_credentials)
        .unwrap();

    session.insert("passkey_registration_state", state).unwrap();

    Json(ccr)
}

#[post("/passkey/register/finish")]
pub(crate) async fn finish_passkey_registration(
    session: Session,
    identity: Identity,
    application: Data<Application>,
    req: Json<RegisterPublicKeyCredential>,
) -> impl Responder {
    let user_id = Uuid::parse_str(&identity.id().unwrap()).unwrap();
    let mut user = application.user_repository.fetch_user(&user_id).unwrap();

    let state: PasskeyRegistration = session.get("passkey_registration_state").unwrap().unwrap();

    let passkey = application
        .webauthn
        .finish_passkey_registration(&req, &state)
        .unwrap();

    user.update_passkey(passkey);
    application.user_repository.save_user(user);

    session.remove("passkey_registration_state");

    HttpResponse::Created()
}

#[post("/logout")]
async fn logout(identity: Identity) -> impl Responder {
    identity.logout();

    HttpResponse::Ok()
}

#[post("/passkey/authenticate/start")]
async fn start_passkey_authentication(
    session: Session,
    application: Data<Application>,
    email: String,
) -> impl Responder {
    let user = application
        .user_repository
        .fetch_user_by_email(&email)
        .unwrap();

    let allow_credentials: Vec<Passkey> = user.get_passkey().cloned().into_iter().collect();

    let (rcr, state) = application
        .webauthn
        .start_passkey_authentication(&allow_credentials)
        .unwrap();

    session
        .insert("passkey_authentication_state", state)
        .unwrap();

    Json(rcr)
}

#[post("/passkey/authenticate/finish")]
async fn finish_passkey_authentication(
    session: Session,
    application: Data<Application>,
    request: HttpRequest,
    req: Json<PublicKeyCredential>,
) -> impl Responder {
    let state = session
        .get("passkey_authentication_state")
        .unwrap()
        .unwrap();

    let result = application
        .webauthn
        .finish_passkey_authentication(&req, &state)
        .unwrap();

    if let Some(user) = application
        .user_repository
        .fetch_user_by_passkey(result.cred_id())
    {
        Identity::login(&request.extensions(), user.id.to_string()).unwrap();
        HttpResponse::Ok()
    } else {
        HttpResponse::NotFound()
    }
}
