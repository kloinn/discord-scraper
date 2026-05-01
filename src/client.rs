use rand::Rng;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread::sleep;
use tokio::sync::Mutex;

use futures_util::StreamExt;
use futures_util::{SinkExt, TryStreamExt, stream::SplitSink, stream::SplitStream};
use log::{debug, error, info, trace, warn};
use serde::Serialize;
use serde_json::json;
use tokio::net::TcpStream;
use tokio::time::{self, Duration};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{WebSocketStream, connect_async};

use crate::cache::CacheManager;
use crate::message::message_handler;
use crate::packet::{
    ClientState as PacketClientState, DPayload, DiscordGatewayPacket, Presence, Properties,
    U64OrString,
};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;

type WriteSink = SplitSink<WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>, Message>;
type ReadStream = SplitStream<WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>>;

#[derive(Debug)]
struct DiscordClientState {
    token: String,
    heartbeat_interval: Arc<Mutex<u64>>,
    last_seq: Arc<AtomicU64>,
    last_ack_received: Arc<AtomicBool>,
    should_reconnect: Arc<AtomicBool>,
}

impl DiscordClientState {
    fn new(token: String) -> Self {
        Self {
            token,
            heartbeat_interval: Arc::new(Mutex::new(0)),
            last_seq: Arc::new(AtomicU64::new(u64::MAX)),
            last_ack_received: Arc::new(AtomicBool::new(true)),
            should_reconnect: Arc::new(AtomicBool::new(false)),
        }
    }

    fn get_last_seq(&self) -> u64 {
        self.last_seq.load(Ordering::SeqCst)
    }

    fn set_last_seq(&self, seq: u64) {
        self.last_seq.store(seq, Ordering::SeqCst)
    }

    fn should_reconnect(&self) -> bool {
        self.should_reconnect.load(Ordering::SeqCst)
    }

    fn set_reconnect(&self, should_reconnect: bool) {
        self.should_reconnect
            .store(should_reconnect, Ordering::SeqCst)
    }

    fn is_last_ack_received(&self) -> bool {
        self.last_ack_received.load(Ordering::SeqCst)
    }

    fn set_last_ack_received(&self, received: bool) {
        self.last_ack_received.store(received, Ordering::SeqCst)
    }

    async fn set_heartbeat_interval(&self, interval: u64) {
        let mut lock = self.heartbeat_interval.lock().await;
        *lock = interval;
    }

    async fn get_heartbeat_interval(&self) -> u64 {
        *self.heartbeat_interval.lock().await
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum SubscriptionData {
    Bool(bool),
    Array(Vec<String>),
    Map(HashMap<String, SubscriptionData>),
}

async fn send_subscriptions(token: String, write: Arc<Mutex<WriteSink>>, guild_id: String) {
    let lol = rand::thread_rng().gen_range(0..60);
    tokio::time::sleep(Duration::from_secs(lol)).await;

    let name_res = CacheManager::get_guild_name(token.as_str(), guild_id.as_str()).await;

    if name_res.is_err() {
        return;
    }

    let name = name_res.unwrap();

    if name.contains("Unknown") {
        return;
    }

    info!("Subbing to {} - {}", guild_id, name);

    let paki = format!(
        r#"{{"d":{{"subscriptions":{{"{}":{{"activities":true,"channels":{{}},"member_updates":false,"members":[],"thread_member_lists":[],"threads":true,"typing":true}}}}}},"op":37}}"#,
        guild_id.replace("\"", "")
    );

    let mut sink = write.lock().await;

    if let Err(e) = sink.send(Message::Text(paki)).await {
        error!("Error sending sub: {}", e);
        return;
    }
}

async fn handle_gateway_packet(
    token: String,
    client_state: Arc<DiscordClientState>,
    write: Arc<Mutex<WriteSink>>,
    packet: DiscordGatewayPacket,
    raw: String,
) {
    if let Some(U64OrString::U64(seq)) = packet.s {
        client_state.set_last_seq(seq);
    }

    match packet.op {
        1 => {
            info!("Got heartbeat request from Discord: {}", raw);

            let last_seq = client_state.get_last_seq();
            let seq_value = if last_seq == u64::MAX {
                serde_json::Value::Null
            } else {
                serde_json::Value::Number(serde_json::Number::from(last_seq))
            };
            let hb = json!({ "op": 1, "d": seq_value }).to_string();

            let mut sink = write.try_lock().unwrap();

            match sink.send(Message::Text(hb.clone())).await {
                Ok(_) => {
                    info!("SENT RESPONSE");
                }
                Err(e) => {
                    error!("HB: {}", e);
                    client_state.set_reconnect(true);
                }
            }
        }
        10 => {
            if let DPayload::Welcome(w) = packet.d {
                info!("Received HELLO (op 10) with raw data: {}", raw);

                client_state
                    .set_heartbeat_interval(w.heartbeat_interval)
                    .await;
                let interval = client_state.get_heartbeat_interval().await;

                let cs = client_state.clone();
                let writer = write.clone();
                tokio::spawn(async move {
                    let mut interval = time::interval(Duration::from_millis(interval));
                    loop {
                        interval.tick().await;

                        if !cs.is_last_ack_received() {
                            error!("didn't recv an ack");
                            cs.set_reconnect(true);
                            break;
                        }

                        cs.set_last_ack_received(false);
                        let last_seq = cs.get_last_seq();
                        let seq_value = if last_seq == u64::MAX {
                            serde_json::Value::Null
                        } else {
                            serde_json::Value::Number(serde_json::Number::from(last_seq))
                        };

                        let hb = json!({ "op": 1, "d": seq_value }).to_string();

                        match writer.lock().await {
                            mut sink => match sink.send(Message::Text(hb.clone())).await {
                                Ok(_) => {
                                    debug!("Sent scheduled heartbeat successfully");
                                }
                                Err(e) => {
                                    error!("Scheduled heartbeat failed: {}", e);
                                    cs.set_reconnect(true);
                                    break;
                                }
                            },
                        }
                    }
                });

                let identify = DiscordGatewayPacket {
                    op: 2,
                    d: DPayload::Identify {
                        token: client_state.token.clone(),
                        capabilities: 161789,
                        properties: Properties {
                            os: "Windows".into(),
                            browser: "Chrome".into(),
                            device: "".into(),
                            system_locale: "en-us".into(),
                            has_client_mods: false,
                            browser_user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36".into(),
                            browser_version: "136.0.0.0".into(),
                            os_version: "10".into(),
                            referrer: "https://discord.com/login".into(),
                            referring_domain: "discord.com".into(),
                            referrer_current: "".into(),
                            referring_domain_current: "".into(),
                            release_channel: "stable".into(),
                            client_build_number: 396183,
                            client_event_source: None,
                            client_app_state: None,
                            is_fast_connect: false,
                        },
                        presence: Presence {
                            status: "unknown".into(),
                            since: 0,
                            activities: vec![],
                            afk: false,
                        },
                        compress: false,
                        client_state: PacketClientState {
                            guild_versions: Arc::new(std::sync::Mutex::new(HashMap::new())),
                        },
                    },
                    s: None,
                    t: None,
                };

                let id_json = serde_json::to_string(&identify).unwrap();

                let mut sink = write.lock().await;

                if let Err(e) = sink.send(Message::Text(id_json)).await {
                    error!("Identify failed: {}", e);
                } else {
                    info!("Identify sent successfully");
                }
            }
        }
        11 => {
            info!("Discord accepted heartbeat (ACK): {}", raw);
            client_state.set_last_ack_received(true);
        }
        _ => {
            if let Some(U64OrString::String(ref kind)) = packet.t {
                match kind.as_str() {
                    "READY" => {
                        info!("READY for {}", client_state.token);

                        if let DPayload::Unknown(p) = packet.d {
                            let guilds = p["guilds"].as_array().unwrap();

                            info!("Got {} guilds", guilds.len());

                            for guild in guilds {
                                let id = &guild["id"];
                                tokio::spawn(send_subscriptions(
                                    token.clone(),
                                    write.clone(),
                                    id.to_string().replace("\"", ""),
                                ));
                            }
                        }
                    }
                    "MESSAGE_CREATE" => {
                        if let DPayload::MessageCreate(msg) = packet.d {
                            message_handler(client_state.token.clone(), &&msg).await;
                        } else {
                            warn!("something is really fucked up")
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

async fn _connect_client(token: String) {
    Box::pin(async move {
        info!("Connecting to Discord Gateway");

        let client_state = Arc::new(DiscordClientState::new(token.clone()));

        let request = "wss://gateway.discord.gg/?v=9&encoding=json"
            .into_client_request()
            .unwrap();

        let connect_result = connect_async(request).await;
        if let Err(e) = connect_result {
            error!("WebSocket connection failed: {}", e);
            return;
        }

        let (ws_stream, _) = connect_result.unwrap();
        info!("WebSocket connection established");

        let (write, mut read): (WriteSink, ReadStream) = ws_stream.split();
        let write = Arc::new(Mutex::new(write));

        while let Ok(Some(msg)) = read.try_next().await {
            if let Ok(text) = msg.into_text() {
                if let Ok(packet) = serde_json::from_str::<DiscordGatewayPacket>(&text) {
                    handle_gateway_packet(
                        token.clone(),
                        client_state.clone(),
                        write.clone(),
                        packet,
                        text,
                    )
                    .await;

                    if client_state.should_reconnect() {
                        client_state.set_reconnect(false);
                        info!(
                            "Reconnection requested, closing current connection and reconnecting..."
                        );
                    }
                } else {
                    warn!("Parse failed: {}", text);
                }
            } else {
                warn!("Received non-text message");
            }
        }
    })
    .await
}

pub async fn connect_client(token: String) {
    loop {
        _connect_client(token.clone()).await;
        info!("Reconnecting");
    }
}
