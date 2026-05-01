use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use log::warn;
use onig::Regex;

static BANS: LazyLock<Mutex<HashMap<String, u64>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
static ATTACHMENT_BANS: LazyLock<Mutex<HashMap<String, u64>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

static PEDO_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    return Regex::new("(?i)(?<!\\w)age\\s*([6-9]|1[0-7])(?!\\d)\\b|(?<!\\w)age\\s*:\\s*([6-9]|1[0-7])(?!\\d)\\b|([6-9]|1[0-7])\\s*-\\s*(\\d{1,2})(?=\\s*$|\\b)|\\b\\s*(?:c4c|t33n|touch|lolicon|touching|rape\\s*children|rape|rape\\s*kids|raping|molest|pedo\\s*mom|molesting|molester|love\\s*kids|love\\s*kiddy)\\b|\\b(p\\s*e\\s*d\\s*o)\\b|\\b(?:sell|trade|buy|exchange|deal|offer|market|auction|purchase|distribute|swap)\\b|https?:\\/\\/(?:[a-zA-Z0-9\\-]+\\.)?(?:t\\.me|telegram\\.me|tme|drive\\.google\\.com|gofile\\.io|mega\\.nz|dropbox\\.com|mediafire\\.com|weTransfer\\.com)\\S*|\\b(?:years?|birthday|youth|old|teen|kids|free\\s*cp|child|baby|leak|leaks|mega|megas|link|links|mega\\s*link|mega\\s*links|adult|senior|elderly|minor|underage|pre-teen|preteen|age\\s*restriction|kiddy)\\b|\\b(\\d{1,2}\\s?(?:yo|years?\\s?old))\\b|(?:https?:\\/\\/)?(?:www\\.)?(?:[a-zA-Z0-9\\-]+\\.)?(?:file\\.io|zippyshare\\.com|dropbox\\.com|mediafire\\.com|mega\\.nz|anonfiles\\.com|filefactory\\.com|transfer\\.sh|wefile\\.io|sendspace\\.com|vfile\\.io|fileup\\.io)\\b(?:\\/[^\\s]*)?|(?:https?:\\/\\/)?(?:www\\.)?(?:[a-z2-7]{16}|[a-z2-7]{56})\\.onion\\b(?:\\/[^\\s]*)?|\\b[sS]elling\\b|\\b[tT]rape{1,}\\b|\\b[lL]eak\\s*[mM]ega\\b|lists|\\b(?:ask\\s*me\\s*privately|message\\s*me\\s*on|personal\\s*chat|dm\\s*me|contact\\s*me\\s*on)\\b|^(?:.*\\b(?:__DISABLED__)\\b.*)$").unwrap();
});

const BAN_DURATION: u64 = 5 * 60;

pub fn is_banned(user_id: &str) -> bool {
    let map = BANS.lock().unwrap();
    if let Some(&ban_time) = map.get(user_id) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now.saturating_sub(ban_time) <= BAN_DURATION
    } else {
        false
    }
}

pub fn ban(user_id: String) {
    warn!("Banned {}", user_id);

    let ban_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut map = BANS.lock().unwrap();
    map.insert(user_id, ban_time);
}


pub fn is_attachment_banned(user_id: &str) -> bool {
    let map = ATTACHMENT_BANS.lock().unwrap();
    if let Some(&ban_time) = map.get(user_id) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now.saturating_sub(ban_time) <= BAN_DURATION
    } else {
        false
    }
}

pub fn attachment_ban(user_id: String) {
    warn!("Attachment banned {}", user_id);

    let ban_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut map = ATTACHMENT_BANS.lock().unwrap();
    map.insert(user_id, ban_time);
}

pub fn is_bad_message(message_content: String) -> bool {
    PEDO_REGEX.is_match(&message_content.as_str())
}

pub fn is_bad_username(username: String) -> bool {
    let username = username.to_lowercase();
    let mut count = 0;

    if username.contains("leak") && !username.contains("leaky") {
        count += 1;
    }

    if username.contains("link") && 
       !username.contains("linka") && 
       !username.contains("unk") && 
       !username.contains("linkx") {
        count += 1;
    }

    if username.contains("sell") {
        count += 1;
    }

    if username.contains("mega") &&
       !username.contains("megatank") &&
       !username.contains("bit") &&
       !username.contains("creator") &&
       !username.contains("master") &&
       !username.contains("omega") {
        count += 1;
    }

    return count >= 1;
}

pub fn is_bad_username_for_reporting(username: String) -> bool {
    let username = username.to_lowercase();
    let mut count = 0;

    if username.contains("leak") && !username.contains("leaky") {
        count += 1;
    }

    if username.contains("link") && 
       !username.contains("linka") && 
       !username.contains("unk") && 
       !username.contains("linkx") {
        count += 1;
    }

    if username.contains("sell") {
        count += 1;
    }

    if username.contains("mega") &&
       !username.contains("megatank") &&
       !username.contains("bit") &&
       !username.contains("creator") &&
       !username.contains("master") &&
       !username.contains("omega") {
        count += 1;
    }

    return count >= 2;
}
