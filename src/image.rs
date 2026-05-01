use crate::database::{DB_POOL, DbImage};
use crate::schema::images::dsl::images;
use crate::{automod, utils};
use diesel::prelude::*;
use log::{debug, info, warn};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::io::BufRead;
use std::io::{BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::LazyLock;

const MAX_IMAGE_SIZE: usize = 3 * 1024 * 1024;

static GLOBAL_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    let client = Client::new();
    return client;
});

#[derive(Serialize, Deserialize, Debug)]
struct ImageAnalysis {
    hash: String,
    is_nsfw: bool,
    predictions: HashMap<String, f64>,
    ban_reason: String,
}

pub fn compress_image(image_bytes: &[u8]) -> Result<Vec<u8>, String> {
    let mut encoder = lz4::EncoderBuilder::new()
        .level(10)
        .build(Vec::new())
        .map_err(|e| e.to_string())?;
    encoder.write_all(image_bytes).map_err(|e| e.to_string())?;
    let (compressed, result) = encoder.finish();
    result.map_err(|e| e.to_string())?;
    Ok(compressed)
}

pub async fn download_and_process_image(id: &str, url: &str, uid: String) -> Option<String> {
    let mut conn = DB_POOL.get().expect("Failed to get DB connection");

    if automod::is_attachment_banned(&uid) {
        warn!("Skipped a banned attachment for uid {} - url {}", uid, url);

        let image = DbImage {
            id: id.to_string(),
            content: Some("".to_string()),
            removal_reason: Some("This image was uploaded by a banned user".to_string()),
            original_url: Some(url.to_string()),
        };

        let _= diesel::insert_into(images)
            .values(&image)
            .execute(&mut conn);

        return Some(image.id);
    }

    let existing_image = images
        .filter(crate::schema::images::original_url.eq(&url))
        .first::<DbImage>(&mut conn)
        .optional()
        .expect("Failed to query database");

    if existing_image.is_some() {
        return Some(existing_image.unwrap().id);
    }

    info!("Downloading image {} with ID {}", url, id);

    let response = reqwest::get(url).await.ok()?;
    let image_bytes = response.bytes().await.ok()?;

    if image_bytes.len() > MAX_IMAGE_SIZE {
        warn!("Image too large: {} bytes", image_bytes.len());

        let image = DbImage {
            id: id.to_string(),
            content: Some("".to_string()),
            removal_reason: Some("This image is above our file size limit of 3 MB".to_string()),
            original_url: Some(url.to_string()),
        };

        diesel::insert_into(images)
            .values(&image)
            .execute(&mut conn)
            .expect("Failed to insert image");

        return Some(image.id);
    }

    let compressed = compress_image(&image_bytes).ok()?;
    let base64_content_uncompressed = base64::encode(&image_bytes);
    let base64_content = base64::encode(compressed);

    let existing_image = images
        .filter(crate::schema::images::content.eq(&base64_content))
        .first::<DbImage>(&mut conn)
        .optional()
        .expect("Failed to query database");

    if let Some(existing) = existing_image {
        return Some(existing.id);
    }

    let mime_type = infer::get(&image_bytes)
        .map(|info| info.mime_type())
        .unwrap_or("image/webp");

    let payload = json!({
        "imageBase64": format!("data:{};base64,{}", mime_type, base64_content_uncompressed)
    });

    let response = GLOBAL_CLIENT
        .post("http://localhost:6666/")
        .header("Content-Type", "application/json")
        .body(payload.to_string())
        .send()
        .await;

    if response.is_err() {
        warn!("{:}", response.unwrap_err());
        return Some("".to_string());
    }

    let text = response.unwrap().text().await;
    let classification_response_2 = serde_json::from_str::<ImageAnalysis>(text.unwrap().as_str());

    if classification_response_2.is_err() {
        warn!("Failed to download");
        return Some("".to_string());
    }

    let classification_response = classification_response_2.unwrap();

    let mut image = DbImage {
        id: id.to_string(),
        content: Some(base64_content.clone()),
        removal_reason: None,
        original_url: Some(url.to_string()),
    };

    if classification_response.is_nsfw {
        warn!("Image flagged as NSFW: {:?}", classification_response);

        image.content = Some("".to_string());
        image.removal_reason = Some(classification_response.ban_reason);
        image.original_url = Some("".to_string());

        automod::attachment_ban(uid);
    }

    let res = diesel::insert_into(images)
        .values(&image)
        .execute(&mut conn);

    if res.is_err() {
        warn!("{:?}", res);
    }

    Some(image.id)
}

pub async fn start_nsfw_processor() -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        info!("Started NSFW processor");

        let mut child = Command::new("cmd")
            .args(["/C", "cd /d D:\\aconite-v2\\nsfw-filter && bun index.ts"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("failed to execute child");

        let stdout = child.stdout.take().expect("Failed to capture stdout");
        let reader = BufReader::new(stdout);

        for line in reader.lines() {
            let line = line.expect("Failed to read line from stdout");

            if line.contains("Server running on") {
                info!("NSFW FILTER RUNNING");
                break;
            }
        }
    })
}
