use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Instant;

static GUILDS: LazyLock<Mutex<HashMap<String, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static CHANNELS: LazyLock<Mutex<HashMap<String, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub struct CacheManager;

impl CacheManager {
    async fn fetch(url: &str, token: &str) -> Option<bytes::Bytes> {
        let client = reqwest::Client::new();
        let mut headers = HeaderMap::new();

        headers.insert("accept", HeaderValue::from_static("*/*"));
        headers.insert(
            "accept-language",
            HeaderValue::from_static("pl-PL,pl;q=0.9,en-US;q=0.8,en;q=0.7"),
        );
        headers.insert(AUTHORIZATION, HeaderValue::from_str(token).unwrap());
        headers.insert("cache-control", HeaderValue::from_static("no-cache"));
        headers.insert("pragma", HeaderValue::from_static("no-cache"));
        headers.insert("priority", HeaderValue::from_static("u=1, i"));
        headers.insert(
            "sec-ch-ua",
            HeaderValue::from_static(
                "\"Chromium\";v=\"136\", \"Google Chrome\";v=\"136\", \"Not.A/Brand\";v=\"99\"",
            ),
        );
        headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("?0"));
        headers.insert(
            "sec-ch-ua-platform",
            HeaderValue::from_static("\"Windows\""),
        );
        headers.insert("sec-fetch-dest", HeaderValue::from_static("empty"));
        headers.insert("sec-fetch-mode", HeaderValue::from_static("cors"));
        headers.insert("sec-fetch-site", HeaderValue::from_static("same-origin"));
        headers.insert("sec-gpc", HeaderValue::from_static("1"));
        headers.insert(
            "x-debug-options",
            HeaderValue::from_static("bugReporterEnabled"),
        );
        headers.insert("x-discord-locale", HeaderValue::from_static("en-US"));
        headers.insert(
            "Referer",
            HeaderValue::from_static(
                "https://discord.com/channels/174837853778345984/206325275733131264",
            ),
        );
        headers.insert(
            "Referrer-Policy",
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        );

        let res = client.get(url).headers(headers).send().await.ok();

        if res.is_some() {
            let response = res.unwrap().bytes().await.unwrap();
            return Some(response);
        }

        None
    }

    pub async fn get_guild_name(token: &str, id: &str) -> Result<String, String> {
        let name = {
            let guilds = GUILDS.lock().unwrap();
            guilds.get(id).cloned()
        };

        if let Some(name) = name {
            return Ok(name);
        }

        let response_opt = Self::fetch(&format!("https://discord.com/api/v9/guilds/{}", id), token)
            .await;

        if response_opt.is_none() {
            return Err("SKILL ISSUE".into());
        }

        let response = response_opt.unwrap();

        let json: serde_json::Value = serde_json::from_slice(&response).unwrap();

        let name = json["name"].as_str().unwrap_or("Unknown").to_string();

        {
            let mut guilds = GUILDS.lock().unwrap();
            guilds.insert(id.to_string(), name.clone());
        }

        Ok(name)
    }

    pub async fn get_channel_name(token: &str, id: &str) -> Result<String, reqwest::Error> {
        let name = {
            let channels = CHANNELS.lock().unwrap();
            channels.get(id).cloned()
        };

        if let Some(name) = name {
            return Ok(name);
        }

        let response = Self::fetch(
            &format!("https://discord.com/api/v9/channels/{}", id),
            token,
        )
        .await
        .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&response).unwrap();
        let name = json["name"].as_str().unwrap_or("Unknown").to_string();

        {
            let mut channels = CHANNELS.lock().unwrap();
            channels.insert(id.to_string(), name.clone());
        }

        Ok(name)
    }
}
