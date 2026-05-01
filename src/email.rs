use std::sync::{Arc, LazyLock, OnceLock};

use gmail1::hyper_rustls::HttpsConnectorBuilder;
use gmail1::hyper_util::client::legacy::Client as LegacyClient;
use gmail1::hyper_util::rt::TokioExecutor;
use gmail1::yup_oauth2::{InstalledFlowAuthenticator, InstalledFlowReturnMethod};
use gmail1::{Gmail, api::ListMessagesResponse};
use google_gmail1 as gmail1;
use google_gmail1::api::{MessagePart, Scope};
use google_gmail1::common::Hub;
use log::{error, info};

use rustls::crypto::aws_lc_rs;
use tokio::task::JoinHandle;

type RetardedGmail = Gmail<
    hyper_rustls::HttpsConnector<google_gmail1::hyper_util::client::legacy::connect::HttpConnector>,
>;

static GLOBAL_HUB: OnceLock<Option<Arc<RetardedGmail>>> = OnceLock::new();

pub async fn get_last_email() -> String {
    let hub = GLOBAL_HUB.get().unwrap().clone().unwrap();

    match hub.users().messages_list("me").doit().await {
        Ok((_, ListMessagesResponse { messages, .. })) => {
            if let Some(msgs) = messages {
                for msg in msgs {
                    if let Some(id) = msg.id {
                        match hub
                            .users()
                            .messages_get("me", &id)
                            .format("raw")
                            .add_scope(Scope::Gmai)
                            .doit()
                            .await
                        {
                            Ok((_, full_msg)) => {
                                if let Some(raw_data) = full_msg.raw {
                                    let mut msg_content = vec![];

                                    for i in raw_data {
                                        msg_content.push(char::from_u32(i.into()).unwrap());
                                    }

                                    let string: String = msg_content.iter().collect();

                                    return string;
                                }
                            }
                            Err(e) => {
                                error!("Failed to get full message: {:?}", e);
                            }
                        }
                    }
                }
            } else {
                return "NO MSGS".to_owned();
            }
        }
        Err(e) => {
            error!("API error: {:?}", e);
            return "".to_owned();
        }
    }

    return "UNREACHABLE".to_owned();
}


pub fn get_reporting_email() -> String {
    return "discordreporting@duck.com".to_string();
}

pub fn start_email() -> JoinHandle<()> {
    tokio::spawn(async {
        info!("Starting email microsvc...");

        aws_lc_rs::default_provider()
            .install_default()
            .expect("failed to install default CryptoProvider");

        let secret = gmail1::yup_oauth2::read_application_secret("clientsecret.json")
            .await
            .expect("failed to read clientsecret.json");

        let auth =
            InstalledFlowAuthenticator::builder(secret, InstalledFlowReturnMethod::HTTPRedirect)
                .persist_tokens_to_disk("token.json")
                .build()
                .await
                .expect("failed to build authenticator");

        let https = HttpsConnectorBuilder::new()
            .with_native_roots()
            .expect("failed to configure roots")
            .https_or_http()
            .enable_http1()
            .build();

        let client = LegacyClient::builder(TokioExecutor::new()).build(https);

        let hub: RetardedGmail = Gmail::new(client, auth);

        if GLOBAL_HUB.set(Some(Arc::new(hub.clone()))).is_err() {
            return;
        }
    })
}
