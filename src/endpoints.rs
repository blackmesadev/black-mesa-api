use hyper::{Body, Request, Response};

use std::collections::HashMap;

use crate::{
    structs::{Appeal, Config},
    Api,
};

impl Api {
    pub async fn get_guild_list(
        &self,
        _req: Request<Body>,
    ) -> Result<Response<Body>, hyper::Error> {
        return Ok(Response::builder()
            .status(501)
            .body(Body::from("not implemented"))
            .unwrap());
    }

    pub async fn get_guild(
        &self,
        req: Request<Body>,
        guild_id: &str,
    ) -> Result<Response<Body>, hyper::Error> {
        let guild = match self.db.get_guild(guild_id).await {
            Ok(guild) => match guild {
                Some(guild) => guild,
                None => {
                    return Ok(self.not_found().await?);
                }
            },
            Err(e) => {
                tracing::error!("error getting guild: {}", e);
                return Ok(self.internal_server_error().await?);
            }
        };

        self.json_response(req, guild).await
    }

    pub async fn post_guild(
        &self,
        req: Request<Body>,
        guild_id: &str,
    ) -> Result<Response<Body>, hyper::Error> {
        match self.db.guild_exists(guild_id).await {
            Ok(exists) => {
                if exists {
                    let params = match self.parse_query(&req).await?.0 {
                        Some(params) => params,
                        None => {
                            return Ok(Response::builder()
                                .status(409)
                                .body(Body::from("Guild already exists"))
                                .unwrap());
                        }
                    };

                    let overwrite = params.get("overwrite").map(|v| v == "true").unwrap_or(false);

                    if overwrite {
                        match self.db.delete_guild(guild_id).await {
                            Ok(_) => {}
                            Err(e) => {
                                tracing::error!("error deleting guild: {}", e);
                                return Ok(self.internal_server_error().await?);
                            }
                        }
                    } else {
                        return Ok(Response::builder()
                            .status(409)
                            .body(Body::from("Guild already exists"))
                            .unwrap());
                    }
                }
            }
            Err(e) => {
                tracing::error!("error checking if guild exists: {}", e);
                return Ok(self.internal_server_error().await?);
            }
        }

        let body_bytes = match hyper::body::to_bytes(req.into_body()).await {
            Ok(bytes) => bytes,
            Err(e) => {
                tracing::error!("error reading body: {}", e);
                return Ok(self.internal_server_error().await?);
            }
        };

        let config: Config = match serde_json::from_slice(&body_bytes) {
            Ok(config) => config,
            Err(e) => {
                tracing::error!("error parsing body: {}", e);
                return Ok(self.bad_request("invalid body").await?);
            }
        };

        match self.db.create_guild(guild_id, config).await {
            Ok(_) => {
                return Ok(Response::builder().status(201).body(Body::empty()).unwrap());
            }
            Err(e) => {
                tracing::error!("error creating guild: {}", e);
                return Ok(self.internal_server_error().await?);
            }
        }
    }

    pub async fn update_guild(
        &self,
        req: Request<Body>,
        guild_id: &str,
    ) -> Result<Response<Body>, hyper::Error> {
        let (parts, body) = req.into_parts();
        let body_bytes = match hyper::body::to_bytes(body).await {
            Ok(bytes) => bytes,
            Err(e) => {
                tracing::error!("error reading body: {}", e);
                return Ok(self.internal_server_error().await?);
            }
        };

        let old = match self.db.get_guild(guild_id).await {
            Ok(guild) => match guild {
                Some(c) => c,
                None => {
                    return Ok(self.not_found().await?);
                }
            },
            Err(e) => {
                tracing::error!("error getting guild: {}", e);
                return Ok(self.internal_server_error().await?);
            }
        };

        let config: Config = match serde_json::from_slice(&body_bytes) {
            Ok(config) => old.merge_from(config),
            Err(e) => {
                tracing::error!("error parsing config: {}", e);
                return Ok(self.bad_request("invalid config").await?);
            }
        };


        match self.db.update_guild(guild_id, config).await {
            Ok(guild) => {
                if let Some(guild) = guild {
                    return self
                        .json_response(Request::from_parts(parts, Body::empty()), guild.config)
                        .await;
                }
                return self.not_found().await;
            }
            Err(e) => {
                tracing::error!("error updating guild: {}", e);
                return Ok(self.internal_server_error().await?);
            }
        }
    }

    pub async fn delete_guild(
        &self,
        _req: Request<Body>,
        guild_id: &str,
    ) -> Result<Response<Body>, hyper::Error> {
        match self.db.delete_guild(guild_id).await {
            Ok(_) => {
                return Ok(Response::builder().status(204).body(Body::empty()).unwrap());
            }
            Err(e) => {
                tracing::error!("error deleting guild: {}", e);
                return Ok(self.internal_server_error().await?);
            }
        }
    }

    pub async fn options_guild(&self, _req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
        Ok(Response::builder()
            .status(200)
            .header("Allow", "GET, POST, PATCH, DELETE, OPTIONS")
            .header("Access-Control-Allow-Headers", "Content-Type")
            .body(Body::empty())
            .unwrap())
    }

    pub async fn get_appeals(
        &self,
        req: Request<Body>,
        id: &str,
    ) -> Result<Response<Body>, hyper::Error> {
        let query_params = match req.uri().query() {
            Some(query) => {
                let mut params = HashMap::new();
                for param in query.split('&') {
                    let parts: Vec<&str> = param.split('=').collect();
                    if parts.len() != 2 {
                        return self.bad_request("malformed query parameters").await;
                    }
                    params.insert(parts[0], parts[1]);
                }
                params
            }
            None => {
                return self.bad_request("missing query parameters").await;
            }
        };

        let id_type = match query_params.get("id_type") {
            Some(id_type) => *id_type,
            None => "guild",
        };

        match id_type {
            "guild" => {
                let appeals = match self.db.get_appeals(Some(id), None).await {
                    Ok(appeals) => appeals,
                    Err(e) => {
                        tracing::error!("error getting appeals: {}", e);
                        return Ok(self.internal_server_error().await?);
                    }
                };

                self.json_response(req, appeals).await
            }
            "user" => {
                let appeals = match self.db.get_appeals(None, Some(id)).await {
                    Ok(appeals) => appeals,
                    Err(e) => {
                        tracing::error!("error getting appeals: {}", e);
                        return Ok(self.internal_server_error().await?);
                    }
                };

                self.json_response(req, appeals).await
            }
            _ => {
                return self.bad_request("invalid id_type").await;
            }
        }
    }

    pub async fn update_appeal(
        &self,
        req: Request<Body>,
        uuid: &str,
        user_id: &str,
    ) -> Result<Response<Body>, hyper::Error> {
        let (parts, body) = req.into_parts();
        let body_bytes = match hyper::body::to_bytes(body).await {
            Ok(bytes) => bytes,
            Err(e) => {
                tracing::error!("error reading body: {}", e);
                return Ok(self.internal_server_error().await?);
            }
        };

        let appeal: Appeal = match serde_json::from_slice(&body_bytes) {
            Ok(appeal) => appeal,
            Err(e) => {
                tracing::error!("error parsing appeal: {}", e);
                return Ok(self.bad_request("invalid appeal").await?);
            }
        };

        match self.db.update_appeal(uuid, user_id, appeal).await {
            Ok(appeal) => {
                let req = Request::from_parts(parts, Body::empty());
                return self.json_response(req, appeal).await;
            }
            Err(e) => {
                tracing::error!("error updating guild: {}", e);
                return Ok(self.internal_server_error().await?);
            }
        }
    }

    pub async fn create_appeal(
        &self,
        req: Request<Body>,
        guild_id: &str,
        user_id: &str,
    ) -> Result<Response<Body>, hyper::Error> {
        let body_bytes = match hyper::body::to_bytes(req.into_body()).await {
            Ok(bytes) => bytes,
            Err(e) => {
                tracing::error!("error reading body: {}", e);
                return Ok(self.internal_server_error().await?);
            }
        };

        let appeal: Appeal = match serde_json::from_slice(&body_bytes) {
            Ok(appeal) => appeal,
            Err(e) => {
                tracing::error!("error parsing appeal: {}", e);
                return Ok(self.bad_request("invalid appeal").await?);
            }
        };

        if appeal.guild_id != guild_id || appeal.user_id != user_id {
            return self.not_authorized("id mismatch").await;
        }

        match self.db.create_appeal(appeal).await {
            Ok(_) => {
                return Ok(Response::builder().status(201).body(Body::empty()).unwrap());
            }
            Err(e) => {
                tracing::error!("error creating guild: {}", e);
                return Ok(self.internal_server_error().await?);
            }
        }
    }

    pub async fn delete_appeal(
        &self,
        _req: Request<Body>,
        uuid: &str,
        user_id: &str,
    ) -> Result<Response<Body>, hyper::Error> {
        match self.db.delete_appeal(uuid, user_id).await {
            Ok(res) => {
                if res.deleted_count == 0 {
                    return self.not_found().await;
                }
                return Ok(Response::builder().status(204).body(Body::empty()).unwrap());
            }
            Err(e) => {
                tracing::error!("error deleting guild: {}", e);
                return Ok(self.internal_server_error().await?);
            }
        }
    }

    pub async fn options_appeal(
        &self,
        _req: Request<Body>,
    ) -> Result<Response<Body>, hyper::Error> {
        Ok(Response::builder()
            .status(200)
            .header("Allow", "GET, POST, PATCH, DELETE, OPTIONS")
            .body(Body::empty())
            .unwrap())
    }
}
