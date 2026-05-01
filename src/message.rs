use crate::{
    automod,
    cache::CacheManager,
    database::{DB_POOL, DbImage},
    dsa,
    packet::MessageReference,
    schema::images::dsl::images,
    utils,
};
use log::{info, warn};
use std::io::Write;

use crate::{
    database::{DbMessage, MESSAGE_QUEUE},
    image,
    packet::{Member, MessageCreate},
};

fn type_to_str(type_: u8) -> &'static str {
    return match type_ {
        0 => "",
        1 => "Added to the group chat/thread",
        2 => "Removed from the group chat/thread",
        3 => "User began call",
        4 => "GC icon changed",
        5 => "GC/Thread name changed",
        6 => "Message pinned",
        7 => "User joined this server",
        8 => "User boosted this server",
        9 => "User boosted this server (tier 1)",
        10 => "User boosted this server (tier 2)",
        11 => "User boosted this server (tier 3)",
        12 => "User followed a channel (CHANNEL_FOLLOW_ADD)",
        14 => "GUILD_DISCOVERY_DISQUALIFIED",
        15 => "GUILD_DISCOVERY_REQUALIFIED",
        16 => "User created a new thread",
        17 => "GUILD_DISCOVERY_GRACE_PERIOD_INITIAL_WARNING",
        18 => "GUILD_DISCOVERY_GRACE_PERIOD_FINAL_WARNING",
        19 => "",
        20 => "User ran a command",
        22 => "GUILD_INVITE_REMINDER",
        23 => "CONTEXT_MENU_COMMAND",
        24 => "AUTO_MODERATION_ACTION",
        25 => "ROLE_SUBSCRIPTION_PURCHASE",
        26 => "INTERACTION_PREMIUM_UPSELL",
        27 => "Stage started",
        28 => "Stage end",
        29 => "Stage speaker",
        31 => "[Stage topic]",
        32 => "GUILD_APPLICATION_PREMIUM_SUBSCRIPTION",
        36 => "GUILD_INCIDENT_ALERT_MODE_ENABLED",
        37 => "GUILD_INCIDENT_ALERT_MODE_DISABLED",
        38 => "GUILD_INCIDENT_REPORT_RAID",
        39 => "GUILD_INCIDENT_REPORT_FALSE_ALARM",
        44 => "PURCHASE_NOTIFICATION",
        _ => "Unknown event",
    };
}

pub async fn message_handler(token: String, message: &&MessageCreate) -> Option<DbMessage> {
    let deref = *message;

    let message_id = deref.id.clone().unwrap();
    let content = &deref.content;
    let author_id = &deref.author.id;
    let author_server_name = &<std::option::Option<Member> as Clone>::clone(&deref.member)
        .unwrap_or(Member {
            avatar: None,
            banner: None,
            communication_disabled_until: None,
            deaf: false,
            flags: 0,
            joined_at: "".to_string(),
            mute: false,
            nick: None,
            pending: false,
            premium_since: None,
            roles: vec![],
        })
        .nick
        .unwrap_or("".to_string());

    let author_display_name = &deref
        .author
        .global_name
        .clone()
        .unwrap_or(deref.author.username.clone());
    let author_username = &deref.author.username;
    if author_username.contains("Want") {
        return None; // SHUT THE FUCK UP RETARD
    }
    let author_discriminator = &deref.author.discriminator;
    let author_is_bot = deref.author.discriminator.len() > 0;

    let channel_id = &deref.channel_id;
    let type_ = deref.type_field;
    let special_content = type_to_str(type_);

    let timestamp = deref.timestamp.clone();

    let embeds = serde_json::to_string(&deref.embeds).unwrap();

    let attachments = serde_json::to_string(&deref.attachments).unwrap();

    let (
        replied_message_id,
        replied_author_id,
        replied_author_display_name,
        replied_author_user_name,
        replied_author_discriminator,
        replied_author_is_bot,
        replied_content,
    ) = match &deref.referenced_message {
        Some(msg) => {
            let msg_ref = deref.referenced_message.clone().unwrap();

            let content = msg_ref["content"].as_str().unwrap();
            let message_id =
                <std::option::Option<MessageReference> as Clone>::clone(&deref.message_reference)
                    .unwrap()
                    .message_id
                    .clone();
            let author_id = msg_ref["author"]["id"].as_str().unwrap();
            let author_username = msg_ref["author"]["username"].as_str().unwrap();
            let author_global_name = msg_ref["author"]["global_name"]
                .as_str()
                .unwrap_or(author_username);
            let author_discriminator = msg_ref["author"]["discriminator"].as_str().unwrap();
            let is_bot = author_discriminator.len() > 2;
            let special_content = || {
                let id: u8 = msg_ref["type"].as_i64().unwrap() as u8;
                return type_to_str(id);
            };

            let mut actual_content = content;

            if actual_content.len() == 0 {
                actual_content = special_content();
            }

            let mut is_bad = false;

            if automod::is_bad_message(actual_content.to_string())
                || automod::is_bad_username(author_username.to_string())
            {
                warn!("Flagged bad reply");
                is_bad = true;
            }

            if is_bad {
                (
                    Some(message_id.to_string()),
                    Some(author_id.to_string()),
                    Some(author_global_name.to_string()),
                    Some(author_username.to_string()),
                    Some(author_discriminator.to_string()),
                    is_bot,
                    Some(
                        "ACONITE-DELETED: Replied to a message that was breaking our rules"
                            .to_string(),
                    ),
                )
            } else {
                (
                    Some(message_id.to_string()),
                    Some(author_id.to_string()),
                    Some(author_global_name.to_string()),
                    Some(author_username.to_string()),
                    Some(author_discriminator.to_string()),
                    is_bot,
                    Some(actual_content.to_string()),
                )
            }
        }
        _ => (None, None, None, None, None, false, None),
    };

    let guild_id =
        <std::option::Option<std::string::String> as Clone>::clone(&deref.guild_id).unwrap();

    let guild_name = CacheManager::get_guild_name(
        &token,
        &<std::option::Option<std::string::String> as Clone>::clone(&message.guild_id)
            .unwrap()
            .clone(),
    )
    .await
    .unwrap();
    let channel_name = CacheManager::get_channel_name(&token, &message.channel_id)
        .await
        .unwrap();

    if deref.referenced_message.is_some() {
        info!(
            "{} (in reply to {}: {}, in {}): {}",
            author_username,
            replied_author_user_name.clone().unwrap(),
            replied_content.clone().unwrap(),
            format!("{} - {}", guild_name, channel_name),
            content,
        );
    } else {
        info!(
            "{} (in {}): {}",
            author_username,
            format!("{} - {}", guild_name, channel_name),
            content,
        );
    }

    let mut is_flagged = false;

    if automod::is_bad_username_for_reporting(message.author.username.clone()) {
        let author_id = message.author.id.to_string();
        let gid = <std::option::Option<std::string::String> as Clone>::clone(&message.guild_id).unwrap();
        let mid = <std::option::Option<std::string::String> as Clone>::clone(&message.id).unwrap();
        let cid = message.channel_id.clone();

        tokio::spawn(async move {
            dsa::report_user(author_id, "This person is selling CSAM".to_string()).await;
            dsa::report_message(gid, cid, mid, "This person is selling CSAM".to_string()).await;
        });
    }

    if automod::is_bad_message(message.content.clone())
        || automod::is_bad_username(message.author.username.clone())
    {
        warn!("Detected a bad message");
        is_flagged = true;
    }

    let avatar_hash = message.author.avatar.clone().unwrap_or("".to_string());

    let avatar_url = format!(
        "https://cdn.discordapp.com/avatars/{}/{}.webp?size=128",
        message.author.id, avatar_hash
    );

    let mut cdn_id = format!("profile_pic-{}", avatar_hash);
    let mut profile_pic_id: Option<String> = None;
    let is_banned = automod::is_banned(&message.author.id);

    if !is_flagged {
        let mut l_profile_pic_id = None;

        if avatar_hash.len() > 0 {
            l_profile_pic_id =
                image::download_and_process_image(&cdn_id, &avatar_url, message.author.id.clone())
                    .await;
        }

        for f in &deref.attachments {
            info!("Preparing attachment {}", f.proxy_url);

            let cdn_id_new = format!("image-{}-{}", message.author.id.clone(), f.proxy_url);

            image::download_and_process_image(&cdn_id_new, &f.proxy_url, message.author.id.clone())
                .await;
        }

        profile_pic_id = l_profile_pic_id;
    } else {
        cdn_id = format!("profile_pic-internal_deleted");
    }

    let mut message = DbMessage {
        id: Some(message_id),
        guild_name: Some(guild_name),
        channel_name: Some(channel_name),
        author_clan_tag: Some(
            deref
                .author
                .clan
                .clone()
                .unwrap_or(serde_json::Value::String("".to_string()))
                .to_string(),
        ),
        author_profile_pic_cdn_id: profile_pic_id,
        content: Some(message.content.clone()),
        guild_id: Some(guild_id),
        author_id: Some(author_id.to_string()),
        author_display_name: Some(author_display_name.to_string()),
        author_server_name: Some(author_server_name.to_string()),
        author_user_name: Some(author_username.to_string()),
        author_discriminator: Some(author_discriminator.to_string()),
        author_is_bot: Some(false),
        channel_id: Some(channel_id.to_string()),
        type_: Some(type_.into()),
        special_content: Some(special_content.to_string()),
        replied_message_id,
        replied_author_id,
        replied_author_display_name,
        replied_author_user_name,
        replied_author_discriminator,
        replied_author_is_bot: Some(false),
        replied_content,
        _timestamp: Some(timestamp),
        embeds: Some(embeds),
        attachments: Some(attachments),
    };

    if is_flagged {
        message.replied_content = None;
        message.replied_author_id = None;
        message.replied_author_display_name = None;
        message.replied_author_user_name = None;
        message.replied_author_discriminator = None;
        message.replied_message_id = None;
        message.content = Some("".into());

        message.embeds = Some("[]".into());
        message.attachments = Some("[]".into());

        if is_banned {
            message.special_content = Some("ACONITE-DELETED: (Automated) This user has been banned for child safety (or related) reasons".to_string())
        } else {
            message.special_content = Some("ACONITE-DELETED: (Automated) This user flagged our automod, likely for child safety".to_string());

            automod::ban(message.author_id.clone().unwrap());
        }

        MESSAGE_QUEUE.lock().await.push_back(message.clone());
        return None;
    }

    MESSAGE_QUEUE.lock().await.push_back(message.clone());

    Some(message)
}
