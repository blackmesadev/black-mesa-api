use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::{errors::Error, Api};

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    pub discriminator: String,
    pub global_name: Option<String>,
    pub avatar: Option<String>,
}

impl Api {
    pub async fn discord_token_to_user(&self, token: &str) -> Result<DiscordUser, Error> {
        let resp = self
            .client
            .get("https://discord.com/api/users/@me")
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        let resp = resp.json::<DiscordUser>().await?;

        Ok(resp)
    }
}
