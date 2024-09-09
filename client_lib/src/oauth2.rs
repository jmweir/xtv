use oauth2::{
    AuthorizationCode,
    AuthUrl,
    ClientId,
    ClientSecret,
    CsrfToken,
    EmptyExtraTokenFields,
    PkceCodeChallenge,
    PkceCodeVerifier,
    RedirectUrl,
    RefreshToken,
    StandardTokenResponse,
    TokenResponse,    
    TokenUrl,
    basic::{
        BasicClient,
        BasicTokenType
    },
    reqwest::async_http_client
};
use actix_web::{
    dev::Server,
    web,
    HttpResponse,
    HttpResponseBuilder
};
use serde::{Deserialize,Serialize};
use rand::{
    thread_rng,
    Rng
};
use std::time::SystemTime;
use std::sync::mpsc;
use chrono::{
    offset::Local,
    DateTime
};
use super::server;
use base64::{Engine as _, engine::general_purpose};

struct AuthAppState {
    client: BasicClient,
    challenge: String
}

#[derive(Deserialize)]
struct TokenRequest {
    code: String
}

#[derive(Clone,Debug,Deserialize,Serialize)]
pub struct Config {
    auth_host: String,
    redirect: String,
    creds: ClientCredentials
}

#[derive(Clone,Debug,Deserialize,Serialize)]
pub struct ClientCredentials {
    client_id: String,
    client_secret: String
}

#[derive(Clone,Debug,Deserialize,Serialize)]
pub struct Token {
    access: String,
    refresh: String,
    expiry: DateTime<Local>
}

impl Token {
    fn from(token_res: StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>) -> Self {
        let expiry = DateTime::<Local>::from(SystemTime::now() + token_res.expires_in().unwrap());
        Token {
            access: token_res.access_token().secret().to_string(),
            refresh: token_res.refresh_token().unwrap().secret().to_string(),
            expiry
        }
    }

    pub fn access(&self) -> &String {
        &self.access
    }

    pub fn refresh(&self) -> &String {
        &self.refresh
    }

    pub fn is_expired(&self) -> bool {
        Local::now() > self.expiry
    }
}

pub async fn authenticate(config: &Config) -> Result<Token, Box<dyn std::error::Error>> {
    let client = client(config)?;

    let challenge = new_random_challenge();

    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .set_pkce_challenge(PkceCodeChallenge::from_code_verifier_sha256(&PkceCodeVerifier::new(challenge.clone())))
        .url();
    
    let (tx, rx) = mpsc::channel::<Token>();

    let server = auth_server(("127.0.0.1", 8080), client, challenge, tx)?;

    open::that(auth_url.to_string())?;

    server.await.map_err(Box::<dyn std::error::Error>::from)?;

    let token = rx.recv()?;

    Ok(token)
}

pub async fn refresh(config: &Config, token: String) -> Result<Token, Box<dyn std::error::Error>> {
    let client = client(config)?;

    let token_response = client
        .exchange_refresh_token(&RefreshToken::new(token))
        .request_async(async_http_client)
        .await?;

    Ok(Token::from(token_response))
}

fn new_random_challenge() -> String {
    let random_bytes: Vec<u8> = (0..32).map(|_| thread_rng().gen::<u8>()).collect();
    general_purpose::STANDARD.encode(&random_bytes)
}

fn auth_server(addrs: impl std::net::ToSocketAddrs, client: BasicClient, challenge: String, tx: mpsc::Sender<Token>) -> Result<Server, Box<dyn std::error::Error>> {
    server::run(addrs, {
        let state = web::Data::new(AuthAppState {
            client,
            challenge
        });
        move |config: &mut web::ServiceConfig| {
            config
                .app_data(state.clone())
                .app_data(web::Data::new(tx.clone()))
                .route("/auth", web::get().to(exchange_code));
        }
    })
}

async fn exchange_code(query: web::Query<TokenRequest>, state: web::Data<AuthAppState>, tx: web::Data<mpsc::Sender<Token>>, stop_handle: web::Data<server::StopHandle>) -> Result<HttpResponseBuilder, Box<dyn std::error::Error>> {
    let token_response = state.client
        .exchange_code(AuthorizationCode::new(query.code.clone()))
        .set_pkce_verifier(PkceCodeVerifier::new(state.challenge.clone()))
        .request_async(async_http_client)
        .await?;

    tx.send(Token::from(token_response))?;

    stop_handle.stop(true);

    Ok(HttpResponse::Ok())
}

fn client(config: &Config) -> Result<BasicClient, Box<dyn std::error::Error>> {
    Ok(
        BasicClient::new(
            ClientId::new(config.creds.client_id.clone()),
            Some(ClientSecret::new(config.creds.client_secret.clone())),
            AuthUrl::new(format!("https://{}/oauth/authorize", config.auth_host.clone()))?,
            Some(TokenUrl::new(format!("https://{}/oauth/token", config.auth_host.clone()))?)
        )
        .set_redirect_uri(RedirectUrl::new(config.redirect.clone())?)
    )
}
