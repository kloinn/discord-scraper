mod automod;
mod cache;
mod client;
mod database;
mod dsa;
mod email;
mod image;
mod message;
mod packet;
mod schema;
mod utils;

use std::{sync::LazyLock, thread::sleep, time::Duration};

use database::start_processing_queue;
use dotenvy::dotenv;
use email::start_email;
use image::start_nsfw_processor;
use log::info;

static TOKENS: LazyLock<Vec<String>> = LazyLock::new(|| {
    let auth_tokens = vec![
    ];

    return auth_tokens;
});

#[tokio::main]
pub async fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .filter_module("reqwest", log::LevelFilter::Off)
        .filter_module("tokio_tungstenite", log::LevelFilter::Off)
        .filter_module("tungstenite", log::LevelFilter::Off)
        .filter_module("yup_oauth2", log::LevelFilter::Off)
        .filter_module("rustls", log::LevelFilter::Off)
        .filter_module("hyper_rustls", log::LevelFilter::Off)
        .init();

    dotenv().ok().expect("Failed to load env");

    info!("Init");

    let mut handles: Vec<tokio::task::JoinHandle<()>> = vec![];

    let mut i = 0;

    handles.push(start_email());
    handles.push(start_nsfw_processor().await);

    for token in TOKENS.iter() {
        let token = token.to_string();

        let handle = {
            let token_clone = token.clone();

            tokio::spawn(async move {
                sleep(Duration::from_secs(i * 20));

                info!("Spawned thread for auth token {}", token_clone);

                client::connect_client(token_clone).await;

                i += 1;
            })
        };

        handles.push(handle);
    }

    handles.push(start_processing_queue());

    for handle in handles {
        let _ = handle.await;
    }
}
