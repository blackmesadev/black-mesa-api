use std::collections::HashMap;

use mongodb::options::UpdateModifications;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Guild {
    pub config: Config,
    pub guild_id: String,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub prefix: Option<String>,
    pub users: Option<HashMap<String, User>>,
    pub roles: Option<HashMap<String, Role>>,
    pub groups: Option<HashMap<String, Group>>,
    pub modules: Option<Modules>,
}

impl Config {
    pub fn merge_from(self, new: Self) -> Self {
        let mut config = self;

        if let Some(prefix) = new.prefix {
            config.prefix = Some(prefix);
        }

        if let Some(users) = new.users {
            config.users = Some(users);
        }

        if let Some(roles) = new.roles {
            config.roles = Some(roles);
        }

        if let Some(groups) = new.groups {
            config.groups = Some(groups);
        }

        if let Some(modules) = new.modules {
            if let Some(old) = config.modules {
                config.modules = Some(old.merge_from(modules));
            } else {
                config.modules = Some(modules);
            }
        }

        config
    }
}

impl From<Config> for bson::Bson {
    fn from(config: Config) -> Self {
        bson::to_bson(&config).unwrap()
    }
}

impl From<Config> for UpdateModifications {
    fn from(config: Config) -> Self {
        UpdateModifications::Document(bson::to_document(&config).unwrap())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub groups: Vec<String>,
    pub permissions: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Role {
    pub groups: Vec<String>,
    pub permissions: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Group {
    pub permissions: Vec<String>,
    pub inherit: Vec<String>,
    pub priority: u64,
}

#[skip_serializing_none]
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Modules {
    pub antinuke: Option<Antinuke>,
    pub antiraid: Option<Antiraid>,
    pub appeals: Option<Appeals>,
    pub automod: Option<Automod>,
    pub logging: Option<Logging>,
    pub moderation: Option<Moderation>,
}

impl Modules {
    pub fn merge_from(self, new: Self) -> Self {
        let mut modules = self;

        if let Some(antinuke) = new.antinuke {
            if let Some(old) = modules.antinuke {
                modules.antinuke = Some(old.merge_from(antinuke));
            } else {
                modules.antinuke = Some(antinuke);
            }
        }

        if let Some(antiraid) = new.antiraid {
            if let Some(old) = modules.antiraid {
                modules.antiraid = Some(old.merge_from(antiraid));
            } else {
                modules.antiraid = Some(antiraid);
            }
        }

        if let Some(appeals) = new.appeals {
            if let Some(old) = modules.appeals {
                modules.appeals = Some(old.merge_from(appeals));
            } else {
                modules.appeals = Some(appeals);
            }
        }

        if let Some(automod) = new.automod {
            if let Some(old) = modules.automod {
                modules.automod = Some(old.merge_from(automod));
            } else {
                modules.automod = Some(automod);
            }
        }

        if let Some(logging) = new.logging {
            if let Some(old) = modules.logging {
                modules.logging = Some(old.merge_from(logging));
            } else {
                modules.logging = Some(logging);
            }
        }

        if let Some(moderation) = new.moderation {
            if let Some(old) = modules.moderation {
                modules.moderation = Some(old.merge_from(moderation));
            } else {
                modules.moderation = Some(moderation);
            }
        }

        modules
    }
}

#[skip_serializing_none]
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Antinuke {
    pub enabled: Option<bool>,
    pub action: Option<AntinukeAction>,
    pub allow_bypass: Option<bool>,
    pub bypass_ids: Option<Vec<String>>,
    pub triggers: Option<Vec<Trigger>>,
}

impl Antinuke {
    pub fn merge_from(self, new: Self) -> Self {
        let mut antinuke = self;

        if let Some(enabled) = new.enabled {
            antinuke.enabled = Some(enabled);
        }
        
        if let Some(action) = new.action {
            antinuke.action = Some(action);
        }

        if let Some(allow_bypass) = new.allow_bypass {
            antinuke.allow_bypass = Some(allow_bypass);
        }

        if let Some(bypass_ids) = new.bypass_ids {
            antinuke.bypass_ids = Some(bypass_ids);
        }

        if let Some(triggers) = new.triggers {
            antinuke.triggers = Some(triggers);
        }
    
        antinuke
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub enum AntinukeAction {
    Ban,
    Kick,
    RemovePermission,
    #[default]
    None,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Trigger {
    pub trigger: AntinukeTrigger,
    pub count: u32,
    pub time: String,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub enum AntinukeTrigger {
    MessageDelete,
    RoleDelete,
    ChannelDelete,
    MemberBan,
    MemberKick,
    #[default]
    None,
}

#[skip_serializing_none]
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Antiraid {
    pub enabled: Option<bool>,
}

impl Antiraid {
    pub fn merge_from(self, new: Self) -> Self {
        let mut antiraid = self;

        if let Some(enabled) = new.enabled {
            antiraid.enabled = Some(enabled);
        }

        antiraid
    }
}

#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Appeals {
    pub enabled: Option<bool>,
    pub channel_id: Option<String>,
    pub appeal_questions: Option<Vec<AppealContent>>,
}

impl Appeals {
    pub fn merge_from(self, new: Self) -> Self {
        let mut appeals = self;

        if let Some(enabled) = new.enabled {
            appeals.enabled = Some(enabled);
        }

        if let Some(channel_id) = new.channel_id {
            appeals.channel_id = Some(channel_id);
        }

        if let Some(appeal_questions) = new.appeal_questions {
            appeals.appeal_questions = Some(appeal_questions);
        }

        appeals
    }
}

#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AppealContent {
    typ: AppealContentType,
    question: Vec<String>,
    answers: Option<Vec<String>>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub enum AppealContentType {
    #[default]
    WrittenResponse,
    MultipleChoice,
    SingleChoice,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Automod {
    pub enabled: Option<bool>,
    pub censor: Option<Vec<Censor>>,
    pub spam: Option<Vec<Spam>>,
}

impl Automod {
    pub fn merge_from(self, new: Self) -> Self {
        let mut automod = self;

        if let Some(enabled) = new.enabled {
            automod.enabled = Some(enabled);
        }

        if let Some(censor) = new.censor {
            automod.censor = Some(censor);
        }

        if let Some(spam) = new.spam {
            automod.spam = Some(spam);
        }

        automod
    }
}

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Censor {
    pub filter_zalgo: Option<bool>,
    pub filter_invites: Option<bool>,
    pub filter_domains: Option<bool>,
    pub filter_strings: Option<bool>,
    pub filter_ips: Option<bool>,
    pub invites_whitelist: Option<Vec<String>>,
    pub invites_blacklist: Option<Vec<String>>,
    pub domain_whitelist: Option<Vec<String>>,
    pub domain_blacklist: Option<Vec<String>>,
    pub blocked_substrings: Option<Vec<String>>,
    pub blocked_strings: Option<Vec<String>>,
    pub regex: Option<String>,

    pub bypass: Vec<String>,
    pub monitor_channels: Vec<String>,
    pub ignore_channels: Vec<String>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Spam {
    pub interval: Option<i64>,
    pub max_messages: Option<i64>,
    pub max_mentions: Option<i64>,
    pub max_links: Option<i64>,
    pub max_attachments: Option<i64>,
    pub max_emojis: Option<i64>,
    pub max_newlines: Option<i64>,
    pub max_characters: Option<i64>,
    pub max_uppercase_percent: Option<f64>,

    pub bypass: Vec<String>,
    pub monitor_channels: Vec<String>,
    pub ignore_channels: Vec<String>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Logging {
    pub enabled: Option<bool>,
    pub channel_id: Option<String>,
    pub include_events: Option<Vec<Event>>,
    pub ignored_users: Option<Vec<String>>,
    pub ignored_channels: Option<Vec<String>>,
}

impl Logging {
    pub fn merge_from(self, new: Self) -> Self {
        let mut logging = self;

        if let Some(enabled) = new.enabled {
            logging.enabled = Some(enabled);
        }

        if let Some(channel_id) = new.channel_id {
            logging.channel_id = Some(channel_id);
        }

        if let Some(include_events) = new.include_events {
            logging.include_events = Some(include_events);
        }

        if let Some(ignored_users) = new.ignored_users {
            logging.ignored_users = Some(ignored_users);
        }

        if let Some(ignored_channels) = new.ignored_channels {
            logging.ignored_channels = Some(ignored_channels);
        }

        logging
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Event {
    None,
    AutomodCensor,
    AutomodSpam,
    Strike,
    Mute,
    Unmute,
    Kick,
    Ban,
    Unban,
    RemoveAction,
    UpdateAction,

    MessageDelete,
    MessageEdit,
    ChannelDelete,
    ChannelEdit,
    ChannelCreate,

    AuditLog,
    MessageLog,
    ModLog,
    AutomodLog,

    All,
}

#[skip_serializing_none]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Moderation {
    pub default_strike_duration: Option<String>,
    pub display_no_permission: Option<bool>,
    pub mute_role: Option<String>,
    pub notify_actions: Option<bool>,
    pub show_moderator_on_notify: Option<bool>,
    pub strike_escalation: Option<HashMap<String, StrikeEscalation>>,
    pub update_higher_level_action: Option<bool>,
}

impl Moderation {
    pub fn merge_from(self, new: Self) -> Self {
        let mut moderation = self;

        if let Some(censor_searches) = new.default_strike_duration {
            moderation.default_strike_duration = Some(censor_searches);
        }

        if let Some(spam_searches) = new.display_no_permission {
            moderation.display_no_permission = Some(spam_searches);
        }

        if let Some(enabled) = new.mute_role {
            moderation.mute_role = Some(enabled);
        }

        if let Some(enabled) = new.notify_actions {
            moderation.notify_actions = Some(enabled);
        }

        if let Some(enabled) = new.show_moderator_on_notify {
            moderation.show_moderator_on_notify = Some(enabled);
        }

        if let Some(enabled) = new.update_higher_level_action {
            moderation.update_higher_level_action = Some(enabled);
        }

        moderation
    }
}

#[skip_serializing_none]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrikeEscalation {
    #[serde(rename = "type")]
    pub typ: PunishmentType,
    pub duration: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub enum PunishmentType {
    #[default]
    Unknown,
    None,
    Strike,
    Mute,
    Kick,
    Ban,
    Softban,
}
