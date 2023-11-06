use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::errors::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub struct Jwt {
    key: EncodingKey,
    dec_key: DecodingKey,
}

impl Jwt {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            key: EncodingKey::from_secret(secret),
            dec_key: DecodingKey::from_secret(secret),
        }
    }

    pub fn create_jwt(&self, user_id: &str) -> Result<(String, usize), Error> {
        let exp = match chrono::Utc::now().checked_add_days(chrono::Days::new(7)) {
            Some(exp) => exp.timestamp(),
            None => chrono::Utc::now().timestamp() + 604800,
        } as usize;

        let claims = Claims {
            sub: user_id.to_string(),
            exp,
        };

        let jwt = jsonwebtoken::encode(&Header::new(Algorithm::HS256), &claims, &self.key)?;

        Ok((jwt, exp))
    }

    pub fn verify_jwt(&self, jwt: &str) -> Result<Claims, Error> {
        let validation = Validation::new(Algorithm::HS256);

        match jsonwebtoken::decode::<Claims>(jwt, &self.dec_key, &validation) {
            Ok(token) => Ok(token.claims),
            Err(e) => Err(Error::from(e)),
        }
    }
}
