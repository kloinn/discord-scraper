use std::{
    sync::LazyLock,
    thread::sleep,
    time::{Duration, Instant},
};

use futures_util::lock::Mutex;
use gemini_rs::Client;
use log::{debug, info};
use onig::Regex;
use reqwest::header::{
    ACCEPT, ACCEPT_LANGUAGE, AUTHORIZATION, CACHE_CONTROL, CONTENT_TYPE, HeaderMap, HeaderValue,
    PRAGMA, REFERER, REFERRER_POLICY,
};
use serde::Serialize;
use serde_json::Value;
use tokio::task::JoinHandle;

use crate::{TOKENS, email};

#[derive(Serialize)]
struct RequestEmailVerification {
    name: String,
    email: String,
}

async fn _request_otp() -> JoinHandle<std::string::String> {
    return tokio::spawn(async move {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
        headers.insert(
            "accept-language",
            HeaderValue::from_static("en-US,en;q=0.9;q=0.8"),
        );
        headers.insert("cache-control", HeaderValue::from_static("no-cache"));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
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
        headers.insert("authorization", HeaderValue::from_static(&TOKENS[0]));
        headers.insert(
            "x-debug-options",
            HeaderValue::from_static("bugReporterEnabled"),
        );
        headers.insert("x-discord-locale", HeaderValue::from_static("en-US"));
        headers.insert(
            "x-discord-timezone",
            // real...
            HeaderValue::from_static("Europe/Warsaw"),
        );
        headers.insert("user-agent", HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36"));
        headers.insert("x-super-properties", HeaderValue::from_static(""));
        headers.insert("cookie", HeaderValue::from_static("__dcfduid=c1d68b7029c311f0b42c9b3eb0673a3c; __sdcfduid=c1d68b7129c311f0b42c9b3eb0673a3c2847ee7074d2fa8189967ed08ea2953f3e4db0c37abeda8f5c10da5d9c84ce3c; OptanonAlertBoxClosed=2025-05-05T17:27:07.099Z; __cfruid=518a06454a06066c8d4dca57a95aea916843a04c-1747033240; _cfuvid=JulGNEJR9U6KXvJrCjz3VqhwSWcsWQUYwVZ0uUl3uxU-1747033240592-0.0.1.1-604800000; locale=en-US; cf_clearance=vc3R9_desvbYjhc7n2vMQkpInEq0knzXoo6FDUZYjws-1747039881-1.2.1.1-wgM4zGty7e5ITjXFOVF4cBttbAgJVlNh6cJQb9uhX9HT3G1cJhemmAE8kR8fAsxQYrtkNpjWajaKsroiezoq6usCWisX9eX5TLsoJBeIB_7FevQ5CndcAqkKd5KzOL023eo0_33mGkYdtfGW5FFUHXW9NpI7KuZjTjHPn2.Bfh6gRFniooxFvIgVYcq6Rt_Oly8URF3llyIX6gh.igHNBQL9BaRF_2drhSfQec7X9DziVF5lDIPcCMTLJbUQnlrvBst2CfpT5Yt8NzHeOjEEmLv1sV_qr7Nw6gJ0.y.E7LbIdYWn1NC34F__cldpxF4oNpjeh3qY3EsMI1s3STCy4CnykvQMY7CUquMTyorTBjg"));
        headers.insert(
            "Referer",
            HeaderValue::from_static("https://discord.com/report"),
        );
        headers.insert(
            "Referrer-Policy",
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        );

        let body = RequestEmailVerification {
            name: String::from("message_urf"),
            email: String::from(email::get_reporting_email()),
        };

        let client = reqwest::Client::new();
        client
            .post("https://discord.com/api/v9/reporting/unauthenticated/message_urf/code")
            .headers(headers)
            .json(&body)
            .send()
            .await
            .expect("reporting failed");

        let mut code = "".to_string();

        loop {
            sleep(Duration::from_secs(5));

            let mail = email::get_last_email().await;
            let mut retard = false;

            for m in mail.split("\r\n") {
                if m.starts_with("Subject:") {
                    code = m
                        .replace("Subject: Your one-time verification key is ", "")
                        .replace("\"", "");
                    retard = true;
                }
            }

            if retard {
                break;
            }
        }

        return code;
    });
}

async fn _login(code: String) -> String {
    let client = reqwest::Client::new();

    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
    headers.insert(
        ACCEPT_LANGUAGE,
        HeaderValue::from_static("en-US,en;q=0.9,pl;q=0.8"),
    );
    headers.insert(AUTHORIZATION, HeaderValue::from_static(&TOKENS[0]));
    headers.insert(CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(PRAGMA, HeaderValue::from_static("no-cache"));
    headers.insert(
        REFERER,
        HeaderValue::from_static("https://discord.com/report"),
    );
    headers.insert(
        REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    let body = format!(
        r#"{{
        "name": "message_urf",
        "email": "discordreporting@duck.com",
        "code": "{}"
    }}"#,
        code
    );

    let response = client
        .post("https://discord.com/api/v9/reporting/unauthenticated/message_urf/verify")
        .headers(headers)
        .body(body.to_string())
        .send()
        .await
        .expect("error");

    let json: Value = response.json().await.unwrap();

    return json["token"].as_str().unwrap().to_string();
}

pub async fn _report_message(url: String, reason: String, jwt: String) {
    let client = reqwest::Client::new();

    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
    headers.insert(
        ACCEPT_LANGUAGE,
        HeaderValue::from_static("en-US,en;q=0.9,pl;q=0.8"),
    );
    headers.insert(AUTHORIZATION, HeaderValue::from_static(&TOKENS[0]));
    headers.insert(CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(PRAGMA, HeaderValue::from_static("no-cache"));
    headers.insert(
        REFERER,
        HeaderValue::from_static("https://discord.com/report"),
    );
    headers.insert(
        REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    let body = format!(
        r#"{{"version":"1.0","variant":"1","language":"en","breadcrumbs":[60,57,132,73,72],"elements":{{"reporter_country":"PL","dsa_free_text":"{}","reporter_legal_name":"Ania Orgov","reported_message_url":"{}","confirmation_select":["validation"]}},"email_token":"{}","name":"message_urf"}}"#,
        reason, url, jwt
    );

    let req = client
        .post("https://discord.com/api/v9/reporting/unauthenticated/message_urf")
        .headers(headers)
        .body(body.to_string())
        .send()
        .await
        .expect("error");

    debug!("response -> {}", req.text().await.unwrap());
}

static CODE_CACHE: LazyLock<Mutex<Option<(String, Instant)>>> = LazyLock::new(|| Mutex::new(None));
static JWT_CACHE: LazyLock<Mutex<Option<(String, Instant)>>> = LazyLock::new(|| Mutex::new(None));
static REPORTED_CACHE: LazyLock<Mutex<Vec<String>>> = LazyLock::new(|| Mutex::new(vec!["".to_string()]));

async fn get_cached_code() -> Option<String> {
    let cache = CODE_CACHE.lock().await;
    if let Some((ref code, expire_time)) = *cache {
        if expire_time > Instant::now() {
            return Some(code.to_string());
        }
    }
    None
}

async fn set_cached_code(code: String) {
    let mut cache = CODE_CACHE.lock().await;
    *cache = Some((code, Instant::now() + Duration::from_secs(600)));
}

async fn get_cached_jwt() -> Option<String> {
    let cache = JWT_CACHE.lock().await;
    if let Some((ref jwt, expire_time)) = *cache {
        if expire_time > Instant::now() {
            return Some(jwt.to_string());
        }
    }
    None
}

async fn set_cached_jwt(jwt: String) {
    let mut cache = JWT_CACHE.lock().await;
    *cache = Some((jwt, Instant::now() + Duration::from_secs(600)));
}

pub async fn _report_user(user: String, message: String, jwt: String) {
    let client = reqwest::Client::new();

    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
    headers.insert(
        ACCEPT_LANGUAGE,
        HeaderValue::from_static("en-US,en;q=0.9,pl;q=0.8"),
    );
    headers.insert(AUTHORIZATION, HeaderValue::from_static(&TOKENS[0]));
    headers.insert(CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(PRAGMA, HeaderValue::from_static("no-cache"));
    headers.insert(
        REFERER,
        HeaderValue::from_static("https://discord.com/report"),
    );
    headers.insert(
        REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    let body = format!(
        r#"{{"version":"1.0","variant":"1","language":"en","breadcrumbs":[59,21,19,168,73,72],"elements":{{"reporter_country":"PL","reporter_legal_name":"Ania Orgov","reporter_username":"","reported_username":"{}","user_profile_select":["name","photos"],"dsa_free_text":"{}","confirmation_select":["validation"]}},"email_token":"{}","name":"user_urf"}}"#,
        user, message, jwt
    );

    let req = client
        .post("https://discord.com/api/v9/reporting/unauthenticated/user_urf")
        .headers(headers)
        .body(body.to_string())
        .send()
        .await
        .expect("error");

    debug!("response -> {}", req.text().await.unwrap());
}

pub async fn report_message(guild: String, channel: String, message_id: String, reason: String) {
    if REPORTED_CACHE.lock().await.contains(&message_id) {
        return;
    }

    REPORTED_CACHE.try_lock().unwrap().push(message_id.clone());

    let code = match get_cached_code().await {
        Some(code) => code,
        None => {
            let new_code = _request_otp().await.await.unwrap();
            set_cached_code(new_code.clone()).await;
            new_code
        }
    };

    let jwt = match get_cached_jwt().await {
        Some(jwt) => jwt,
        None => {
            let new_jwt = _login(code).await;
            set_cached_jwt(new_jwt.clone()).await;
            new_jwt
        }
    };

    let url = format!(
        "https://discord.com/channels/{}/{}/{}",
        guild, channel, message_id
    );

    info!("Reporting {} for {}", url, reason);

    _report_message(url, reason, jwt).await;
}

pub async fn report_user(id: String, reason: String) {
    if REPORTED_CACHE.lock().await.contains(&id) {
        return;
    }

    REPORTED_CACHE.try_lock().unwrap().push(id.clone());

    let code = match get_cached_code().await {
        Some(code) => code,
        None => {
            let new_code = _request_otp().await.await.unwrap();
            set_cached_code(new_code.clone()).await;
            new_code
        }
    };

    let jwt = match get_cached_jwt().await {
        Some(jwt) => jwt,
        None => {
            let new_jwt = _login(code).await;
            set_cached_jwt(new_jwt.clone()).await;
            new_jwt
        }
    };

    info!("Reporting user {} for {}", id, reason);

    _report_user(id, reason, jwt).await;
}
