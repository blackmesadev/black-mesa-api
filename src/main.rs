use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
    sync::Arc,
};

use errors::Error;
use hyper::{Body, Method, Request, Response};

use crate::mongo::Database;

mod auth;
mod discord;
mod endpoints;
mod errors;
mod middleware;
mod mongo;
mod oauth;
mod structs;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const HOST: &str = "0.0.0.0";
const PORT: u16 = 8080;

pub struct Api {
    config: ApiConfig,
    db: mongo::Database,
    client: reqwest::Client,
    jwt: auth::Jwt,
}

pub struct ApiConfig {
    pub discord_client_id: String,
    pub discord_client_secret: String,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            discord_client_id: std::env::var("DISCORD_CLIENT_ID")
                .expect("DISCORD_CLIENT_ID must be set"),
            discord_client_secret: std::env::var("DISCORD_CLIENT_SECRET")
                .expect("DISCORD_CLIENT_SECRET must be set"),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let secret = std::env::var("JWT_SECRET")
        .expect("JWT_SECRET must be set")
        .bytes()
        .collect::<Vec<u8>>();

    let api = Api {
        config: ApiConfig::default(),
        db: Database::new(&std::env::var("MONGO_URI").expect("MONGO_URI must be set")).await,
        client: reqwest::Client::new(),
        jwt: auth::Jwt::new(&secret),
    };

    tracing::info!("Starting Black Mesa Public REST API v{}", VERSION);

    let api_ref = Arc::new(api);
    let make_svc = hyper::service::make_service_fn(move |_| {
        let api_ref = api_ref.clone();
        async move {
            Ok::<_, hyper::Error>(hyper::service::service_fn(move |req| {
                let api_ref = api_ref.clone();
                async move { api_ref.handle_request(req).await }
            }))
        }
    });

    let addr = SocketAddr::new(
        IpAddr::V4(Ipv4Addr::from_str(HOST).expect("Invalid host specified")),
        PORT,
    );

    let server = hyper::Server::bind(&addr).serve(make_svc);

    tokio::select!(
        _ = server => {},
        _ = async move {
            tracing::info!("Black Mesa Public REST API running on http://{}:{}", HOST, PORT);
            tokio::signal::ctrl_c().await.unwrap();
        } => {

        }
    );

    tracing::info!("Shutting down");

    Ok(())
}

impl Api {
    async fn handle_request(&self, req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
        tracing::info!("{} {}", req.method(), req.uri().path());

        let uri = req.uri().clone();
        let path_parts: Vec<&str> = uri.path().trim_end_matches('/').split('/').collect();

        if let Some(section) = path_parts.get(1) {
            match *section {
                "api" => {
                    if let Some(api_version) = path_parts.get(2) {
                        match *api_version {
                            "v1" => return self.handle_v1(req, path_parts).await,
                            _ => {}
                        }
                    }
                }
                "oauth" => {
                    if let Some(operation) = path_parts.get(2) {
                        match *operation {
                            "callback" => return self.discord_oauth(req).await,
                            "login" => return self.discord_login(req).await,
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        self.not_found().await
    }

    async fn handle_v1(
        &self,
        req: Request<Body>,
        path_parts: Vec<&str>,
    ) -> Result<Response<Body>, hyper::Error> {
        let auth = match self.authenticate(&req).await {
            Ok((resp, auth)) => {
                if let Some(resp) = resp {
                    return Ok(resp);
                }
                match auth {
                    Some(auth) => auth,
                    None => return Ok(self.not_authorized("Unauthorized").await?),
                }
            }
            Err(err) => {
                tracing::error!("Error authenticating request: {}", err);
                return Ok(self.internal_server_error().await?);
            }
        };

        if let Some(endpoint) = path_parts.get(3) {
            return match (*endpoint, req.method()) {
                ("guilds", &Method::GET) => self.get_guild_list(req).await,

                ("guild", _) => match path_parts.get(4) {
                    Some(guild_id) => match req.method() {
                        &Method::GET => self.get_guild(req, *guild_id).await,
                        &Method::POST => self.post_guild(req, *guild_id).await,
                        &Method::PATCH => self.update_guild(req, *guild_id).await,
                        &Method::DELETE => self.delete_guild(req, *guild_id).await,
                        _ => self.method_not_allowed().await,
                    },
                    None => match req.method() {
                        &Method::GET => self.get_guild_list(req).await,

                        _ => self.method_not_allowed().await,
                    },
                },
                ("appeals", _) => match path_parts.get(4) {
                    Some(id) => match req.method() {
                        &Method::GET => self.get_appeals(req, *id).await,
                        &Method::POST => self.create_appeal(req, *id, &auth.sub).await,
                        &Method::PATCH => self.update_appeal(req, *id, &auth.sub).await,
                        &Method::DELETE => self.delete_appeal(req, *id, &auth.sub).await,
                        &Method::OPTIONS => self.options_appeal(req).await,
                        _ => self.method_not_allowed().await,
                    },
                    None => self.bad_request("an id must be specified").await,
                },

                _ => self.not_found().await,
            };
        }

        self.not_found().await
    }

    pub async fn parse_query(
        &self,
        req: &Request<Body>,
    ) -> Result<(Option<HashMap<String, String>>, Option<Response<Body>>), hyper::Error> {
        match req.uri().query() {
            Some(query) => {
                let mut params = HashMap::new();
                for param in query.split('&') {
                    let parts: Vec<&str> = param.split('=').collect();
                    if parts.len() != 2 {
                        return Ok((
                            None,
                            Some(self.bad_request("invalid query parameters").await?),
                        ));
                    }
                    params.insert(parts[0].to_string(), parts[1].to_string());
                }
                return Ok((Some(params), None));
            }
            None => {
                return Ok((
                    None,
                    Some(self.bad_request("missing query parameters").await?),
                ));
            }
        }
    }

    async fn not_found(&self) -> Result<Response<Body>, hyper::Error> {
        Ok(Response::builder()
            .status(404)
            .body(Body::from("Not found"))
            .unwrap())
    }

    async fn not_authorized(&self, msg: &str) -> Result<Response<Body>, hyper::Error> {
        Ok(Response::builder()
            .status(401)
            .body(Body::from(format!("Not authorized: {}", msg)))
            .unwrap())
    }

    async fn method_not_allowed(&self) -> Result<Response<Body>, hyper::Error> {
        Ok(Response::builder()
            .status(405)
            .body(Body::from("Method not allowed"))
            .unwrap())
    }

    async fn bad_request(&self, msg: &str) -> Result<Response<Body>, hyper::Error> {
        Ok(Response::builder()
            .status(400)
            .body(Body::from(format!("Bad request: {}", msg)))
            .unwrap())
    }

    async fn internal_server_error(&self) -> Result<Response<Body>, hyper::Error> {
        Ok(Response::builder()
            .status(500)
            .body(Body::from("Internal server error"))
            .unwrap())
    }

    async fn json_response<T>(
        &self,
        _req: Request<Body>,
        json: T,
    ) -> Result<Response<Body>, hyper::Error>
    where
        T: serde::Serialize,
    {
        let json = match serde_json::to_string(&json) {
            Ok(json) => json,
            Err(_) => return Ok(self.internal_server_error().await?),
        };

        Ok(Response::builder()
            .header("Content-Type", "application/json")
            .header("Content-Length", json.len())
            .body(Body::from(json))
            .unwrap())
    }
}
