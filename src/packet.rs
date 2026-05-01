use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::ser::SerializeStruct;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WelcomePacket {
    pub op: u8,
    pub d: PacketData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PacketData {
    pub token: String,
    pub capabilities: u32,
    pub properties: Properties,
    pub presence: Presence,
    pub compress: bool,
    pub client_state: ClientState,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Properties {
    pub os: String,
    pub browser: String,
    pub device: String,
    pub system_locale: String,
    pub has_client_mods: bool,
    pub browser_user_agent: String,
    pub browser_version: String,
    pub os_version: String,
    pub referrer: String,
    pub referring_domain: String,
    pub referrer_current: String,
    pub referring_domain_current: String,
    pub release_channel: String,
    pub client_build_number: u32,
    pub client_event_source: Option<String>,
    pub client_app_state: Option<String>,
    pub is_fast_connect: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Presence {
    pub status: String,
    pub since: u64,
    pub activities: Vec<String>,
    pub afk: bool,
}

#[derive(Debug, Clone)]
pub struct ClientState {
    pub guild_versions: Arc<Mutex<HashMap<String, String>>>,
}

impl Serialize for ClientState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let guild_versions = self.guild_versions.lock().unwrap();
        let mut state = serializer.serialize_struct("ClientState", 1)?;
        state.serialize_field("guild_versions", &*guild_versions)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for ClientState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let map = HashMap::<String, String>::deserialize(deserializer)?;
        Ok(ClientState {
            guild_versions: Arc::new(Mutex::new(map)),
        })
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum U64OrString {
    U64(u64),
    String(String),
}

fn deserialize_u64_or_string<'de, D>(deserializer: D) -> Result<Option<U64OrString>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Value = Deserialize::deserialize(deserializer)?;

    match value {
        Value::Number(num) if num.is_u64() => Ok(Some(U64OrString::U64(num.as_u64().unwrap()))),
        Value::String(s) => Ok(Some(U64OrString::String(s))),
        Value::Null => Ok(None),
        _ => Err(serde::de::Error::custom(format!(
            "Expected a U64 or String, but got {:?}",
            value
        ))),
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DiscordGatewayPacket {
    #[serde(deserialize_with = "deserialize_u64_or_string")]
    pub t: Option<U64OrString>,
    #[serde(deserialize_with = "deserialize_u64_or_string")]
    pub s: Option<U64OrString>,
    pub op: i32,
    pub d: DPayload,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Author {
    pub avatar: Option<String>,
    pub avatar_decoration_data: Option<Value>,
    pub clan: Option<Value>,
    pub collectibles: Option<Value>,
    pub discriminator: String,
    pub global_name: Option<String>,
    pub id: String,
    pub primary_guild: Option<Value>,
    pub public_flags: u64,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Member {
    pub avatar: Option<Value>,
    pub banner: Option<Value>,
    pub communication_disabled_until: Option<Value>,
    pub deaf: bool,
    pub flags: u64,
    pub joined_at: String,
    pub mute: bool,
    pub nick: Option<String>,
    pub pending: bool,
    pub premium_since: Option<String>,
    pub roles: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WelcomePakiData {
    pub heartbeat_interval: u64,
    pub _trace: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageReference {
    #[serde(rename = "type")]
    pub type_: u8,
    pub channel_id: String,
    pub message_id: String,
    pub guild_id: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageCreate {
    pub application_id: Option<String>,
    pub id: Option<String>, // MSG ID
    pub channel_id: String,
    pub author: Author,
    pub content: String,
    pub timestamp: String,
    pub edited_timestamp: Option<String>,
    pub tts: bool,
    pub mention_everyone: bool,
    pub mentions: Vec<Author>,
    pub mention_roles: Vec<String>,
    pub mention_channels: Option<Vec<String>>,
    pub attachments: Vec<Attachment>,
    pub embeds: Vec<serde_json::Value>,
    pub reactions: Option<Vec<serde_json::Value>>,
    pub pinned: bool,
    pub webhook_id: Option<String>,
    #[serde(rename = "type")]
    pub type_field: u8,
    pub member: Option<Member>,
    pub guild_id: Option<String>,
    pub channel_type: u8,
    pub components: Vec<serde_json::Value>,
    pub flags: u64,
    pub nonce: Option<String>,
    pub message_reference: Option<MessageReference>,
    pub referenced_message: Option<Value>
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Attachment {
    pub id: String,
    pub filename: String,
    pub size: u64,
    #[serde(rename = "type")]
    pub embed_type: Option<String>,
    pub content_type: Option<String>,
    pub content_scan_version: Option<u8>,
    pub thumbnail: Option<Thumbnail>,
    pub flags: Option<u8>,
    pub height: Option<u32>,
    pub width: Option<u32>,
    pub placeholder: Option<String>,
    pub placeholder_version: Option<u8>,
    pub proxy_url: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Thumbnail {
    pub height: u32,
    pub width: u32,
    pub url: String,
}


#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DPayload {
    Welcome(WelcomePakiData),
    Identify {
        token: String,
        capabilities: u64,
        properties: Properties,
        presence: Presence,
        compress: bool,
        client_state: ClientState,
    },
    MessageCreate(MessageCreate),
    Unknown(serde_json::Value),
}
