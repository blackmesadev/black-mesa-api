use core::fmt;

use mongodb::options::UpdateModifications;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::AppealContent;

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Appeal {
    pub guild_id: String,
    pub user_id: String,
    pub punishment_uuid: String,
    pub content: Vec<AppealContent>,
    pub status: AppealStatus,
    pub status_reason: Option<String>,
}

impl From<Appeal> for bson::Bson {
    fn from(appeal: Appeal) -> Self {
        bson::to_bson(&appeal).unwrap()
    }
}

impl From<Appeal> for UpdateModifications {
    fn from(appeal: Appeal) -> Self {
        UpdateModifications::Document(bson::to_document(&appeal).unwrap())
    }
}

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AppealStatus {
    Pending,
    Approved,
    Denied,
}

impl fmt::Display for AppealStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppealStatus::Pending => write!(f, "pending"),
            AppealStatus::Approved => write!(f, "approved"),
            AppealStatus::Denied => write!(f, "denied"),
        }
    }
}

impl Appeal {
    pub fn new(
        guild_id: String,
        user_id: String,
        punishment_uuid: String,
        content: Vec<AppealContent>,
    ) -> Self {
        Self {
            guild_id,
            user_id,
            punishment_uuid,
            content,
            status: AppealStatus::Pending,
            status_reason: None,
        }
    }
}
