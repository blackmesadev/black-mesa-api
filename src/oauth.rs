use hyper::{Body, Request, Response};
use serde::{Deserialize, Serialize};

use crate::{errors::Error, Api};

#[derive(Debug, Serialize, Deserialize)]
pub struct OauthResponse {
    pub token: String, // this is a black mesa api
    pub expires: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token: String,
    pub scope: String,
}

impl Api {
    pub async fn discord_oauth(&self, req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
        let query_params = match self.parse_query(&req).await {
            Ok((query_params, resp)) => {
                if let Some(resp) = resp {
                    return Ok(resp);
                }

                query_params.unwrap() // safe to unwrap because resp and query_params can not both be None
            }
            Err(err) => {
                tracing::error!("Error parsing query params: {}", err);
                return self.internal_server_error().await;
            }
        };

        let code = match query_params.get("code") {
            Some(code) => code,
            None => return self.bad_request("missing code parameter").await,
        };

        let access_token = match self.exchange_discord_code(code, &make_callback_uri(&req).await).await {
            Ok(resp) => resp.access_token,
            Err(err) => {
                tracing::error!("Error exchanging discord code: {}", err);
                return self.internal_server_error().await;
            }
        };

        let discord_user = match self.discord_token_to_user(&access_token).await {
            Ok(user) => user,
            Err(err) => {
                tracing::error!("Error getting discord user: {}", err);
                return self.internal_server_error().await;
            }
        };

        let (jwt, exp) = match self.jwt.create_jwt(&discord_user.id) {
            Ok(jwt) => jwt,
            Err(err) => {
                tracing::error!("Error creating jwt: {}", err);
                return self.internal_server_error().await;
            }
        };

        let resp = OauthResponse {
            token: jwt,
            expires: exp,
        };

        self.json_response(req, resp).await
    }

    pub async fn discord_login(&self, req: Request<Body>) -> Result<Response<Body>, hyper::Error> {

        let url = format!(
            "https://discord.com/api/oauth2/authorize?client_id={}&redirect_uri={}&response_type=code&scope=identify",
            self.config.discord_client_id,
            make_callback_uri(&req).await
        );

        return Ok(
            Response::builder()
                .status(302)
                .header("Location", url)
                .body(Body::from(""))
                .unwrap(),
        );
    }

    async fn exchange_discord_code(&self, code: &str, redirect_uri: &str) -> Result<AccessTokenResponse, Error> {
        let resp = self
            .client
            .post("https://discord.com/api/oauth2/token")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .basic_auth(
                &self.config.discord_client_id,
                Some(&self.config.discord_client_secret),
            )
            .body(format!(
                "grant_type=authorization_code&code={}&redirect_uri={}",
                code, redirect_uri
            ))
            .send()
            .await?;

        let resp = resp.json::<AccessTokenResponse>().await?;

        Ok(resp)
    }
}

async fn make_callback_uri(req: &Request<Body>) -> String {
    let host = req.uri().host().unwrap_or("localhost");

    format!("http://{}/oauth/callback", host)
}