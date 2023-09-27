use crate::{
    handlers::{
        finish_passkey_authentication, finish_passkey_registration, logout, register_user,
        start_passkey_authentication, start_passkey_registration, Application,
    },
    session::InMemorySessionStore,
};
use actix_identity::IdentityMiddleware;
use actix_session::SessionMiddleware;
use actix_web::{cookie::Key, middleware::Logger, web::Data, App, HttpServer};
use env_logger::Env;
use log::info;
use rustls::{Certificate, PrivateKey, ServerConfig};
use std::{fs::File, io::BufReader};

mod handlers;
mod session;
mod user;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    let application = Data::new(Application::new());
    let session_secret_key = Key::generate();
    let server_config = create_tls_server_config();

    info!("start server at 127.0.0.1:8081");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(IdentityMiddleware::default())
            .wrap(
                SessionMiddleware::builder(InMemorySessionStore, session_secret_key.clone())
                    .cookie_name("passkey-demo".to_string())
                    .build(),
            )
            .app_data(application.clone())
            .service(register_user)
            .service(start_passkey_registration)
            .service(finish_passkey_registration)
            .service(start_passkey_authentication)
            .service(finish_passkey_authentication)
            .service(logout)
    })
    .bind_rustls("passkey-demo.localhost:8081", server_config)?
    .run()
    .await
}

fn create_tls_server_config() -> ServerConfig {
    let certs = {
        let file = File::open("certs/passkey-demo.localhost+2.pem").unwrap();
        let mut reader = BufReader::new(file);
        let certs = rustls_pemfile::certs(&mut reader).unwrap();

        certs.into_iter().map(Certificate).collect()
    };

    let private_key = {
        let file = File::open("certs/passkey-demo.localhost+2-key.pem").unwrap();
        let mut reader = BufReader::new(file);
        let mut keys = rustls_pemfile::pkcs8_private_keys(&mut reader).unwrap();

        match keys.len() {
            0 => panic!("No PKCS8-encoded private key found in path"),
            1 => PrivateKey(keys.remove(0)),
            _ => panic!("More than one PKCS8-encoded private key found in path"),
        }
    };

    ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, private_key)
        .expect("bad certificate/key")
}
