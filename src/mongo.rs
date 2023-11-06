use mongodb::{
    bson::doc,
    options::FindOneAndUpdateOptions,
    results::{DeleteResult, InsertOneResult},
    Client, Collection,
};

use futures_util::StreamExt;

use crate::structs::{Appeal, Config, Guild};

#[derive(Clone, Debug)]
pub struct Database {
    pub client: Client,
}

impl Database {
    pub async fn new(uri: &str) -> Self {
        let client = Client::with_uri_str(uri)
            .await
            .expect("Failed to connect to MongoDB");

        Self { client }
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_guild(&self, guild_id: &str) -> Result<Option<Config>, mongodb::error::Error> {
        let guilds: Collection<Guild> = self.client.database("black-mesa").collection("guilds");

        let res = guilds.find_one(doc! { "guild_id": guild_id }, None).await?;

        match res {
            Some(guild) => Ok(Some(guild.config)),
            None => Ok(None),
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn guild_exists(&self, guild_id: &str) -> Result<bool, mongodb::error::Error> {
        let guilds: Collection<Guild> = self.client.database("black-mesa").collection("guilds");

        let res = guilds.find_one(doc! { "guild_id": guild_id }, None).await?;

        Ok(res.is_some())
    }

    #[tracing::instrument(skip(self))]
    pub async fn create_guild(
        &self,
        guild_id: &str,
        config: Config,
    ) -> Result<InsertOneResult, mongodb::error::Error> {
        let guilds: Collection<Guild> = self.client.database("black-mesa").collection("guilds");

        let res = guilds
            .insert_one(
                Guild {
                    guild_id: guild_id.to_string(),
                    config,
                },
                None,
            )
            .await?;

        Ok(res)
    }

    pub async fn update_guild(
        &self,
        guild_id: &str,
        config: Config,
    ) -> Result<Option<Guild>, mongodb::error::Error> {
        let guilds: Collection<Guild> = self.client.database("black-mesa").collection("guilds");

        let options = FindOneAndUpdateOptions::builder()
            .upsert(true)
            .return_document(mongodb::options::ReturnDocument::After)
            .build();

        let res = guilds
            .find_one_and_update(
                doc! { "guild_id": guild_id },
                doc! { "$set" : { "config" : config} },
                options,
            )
            .await?;

        Ok(res)
    }

    pub async fn delete_guild(
        &self,
        guild_id: &str,
    ) -> Result<Option<DeleteResult>, mongodb::error::Error> {
        let guilds: Collection<Guild> = self.client.database("black-mesa").collection("guilds");

        let res = guilds
            .delete_one(doc! { "guild_id": guild_id }, None)
            .await?;

        Ok(Some(res))
    }

    pub async fn create_appeal(
        &self,
        appeal: Appeal,
    ) -> Result<InsertOneResult, mongodb::error::Error> {
        let appeals: Collection<Appeal> = self.client.database("black-mesa").collection("appeals");

        let res = appeals.insert_one(appeal, None).await?;

        Ok(res)
    }

    pub async fn get_appeals(
        &self,
        guild_id: Option<&str>,
        user_id: Option<&str>,
    ) -> Result<Vec<Appeal>, mongodb::error::Error> {
        let appeals: Collection<Appeal> = self.client.database("black-mesa").collection("appeals");

        let mut filter = doc! {};

        if let Some(guild_id) = guild_id {
            filter.insert("guild_id", guild_id);
        }

        if let Some(user_id) = user_id {
            filter.insert("user_id", user_id);
        }

        let mut res = appeals.find(filter, None).await?;

        let mut appeals = Vec::new();

        while let Some(result) = res.next().await {
            appeals.push(result?);
        }

        Ok(appeals)
    }

    pub async fn delete_appeal(
        &self,
        uuid: &str,
        user_id: &str,
    ) -> Result<DeleteResult, mongodb::error::Error> {
        let appeals: Collection<Appeal> = self.client.database("black-mesa").collection("appeals");

        let res = appeals
            .delete_one(doc! { "id": uuid, "user_id": user_id }, None)
            .await?;

        Ok(res)
    }

    pub async fn update_appeal(
        &self,
        uuid: &str,
        user_id: &str,
        appeal: Appeal,
    ) -> Result<Option<Appeal>, mongodb::error::Error> {
        let appeals: Collection<Appeal> = self.client.database("black-mesa").collection("appeals");

        let options = FindOneAndUpdateOptions::builder()
            .upsert(true)
            .return_document(mongodb::options::ReturnDocument::After)
            .build();

        let res = appeals
            .find_one_and_update(
                doc! { "id": uuid, "user_id": user_id },
                doc! { "$set" : appeal },
                options,
            )
            .await?;

        Ok(res)
    }
}
