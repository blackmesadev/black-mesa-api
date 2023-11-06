use hyper::{Body, Request, Response};

use crate::{auth::Claims, Api};

impl Api {
    pub async fn authenticate(
        &self,
        req: &Request<Body>,
    ) -> Result<(Option<Response<Body>>, Option<Claims>), hyper::Error> {
        let token = req
            .headers()
            .get("Authorization")
            .and_then(|token| token.to_str().ok())
            .and_then(|header| header.strip_prefix("Bearer ").map(|t| t.to_string()));

        let token = match token {
            Some(token) => token,
            None => {
                return Ok((
                    Some(self.not_authorized("invalid authorization header").await?),
                    None,
                ));
            }
        };

        let claims = match self.jwt.verify_jwt(&token) {
            Ok(claims) => claims,
            Err(_) => {
                return Ok((
                    Some(self.not_authorized("invalid authorization header").await?),
                    None,
                ));
            }
        };

        Ok((None, Some(claims)))
    }
}
