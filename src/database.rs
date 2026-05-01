use std::{collections::VecDeque, env, sync::LazyLock, thread, time::Duration};

use crate::schema::messages::dsl::*;
use crate::schema::{self, images, messages};
use diesel::{
    PgConnection, RunQueryDsl, insert_into,
    prelude::{Insertable, Queryable},
    r2d2::{self, ConnectionManager},
};
use tokio::sync::Mutex;
use log::{info, warn};

#[derive(Insertable, Queryable, Debug, Clone)]
#[diesel(table_name = messages)]
pub struct DbMessage {
    pub id: Option<String>,

    pub author_id: Option<String>,
    pub author_display_name: Option<String>,
    pub author_user_name: Option<String>,
    pub author_server_name: Option<String>,

    pub author_profile_pic_cdn_id: Option<String>,
    pub author_clan_tag: Option<String>,

    pub author_discriminator: Option<String>,
    pub author_is_bot: Option<bool>,

    pub channel_id: Option<String>,
    pub channel_name: Option<String>,

    pub guild_id: Option<String>,
    pub guild_name: Option<String>,

    pub content: Option<String>,
    pub special_content: Option<String>,

    pub replied_message_id: Option<String>,

    pub replied_author_id: Option<String>,
    pub replied_author_display_name: Option<String>,
    pub replied_author_user_name: Option<String>,
    pub replied_author_discriminator: Option<String>,
    pub replied_author_is_bot: Option<bool>,

    pub replied_content: Option<String>,

    pub embeds: Option<String>,
    pub attachments: Option<String>,

    pub _timestamp: Option<String>,

    pub type_: Option<i16>,
}


#[derive(Insertable, Queryable, Debug, Clone)]
#[diesel(table_name = images)]
pub struct DbImage {
    pub id: String,
    pub content: Option<String>,
    pub removal_reason: Option<String>,
    pub original_url: Option<String>
}   

type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub static DB_POOL: LazyLock<Pool> = LazyLock::new(|| {
    let database_url = env::var("PG_DATABASE_URL")
        .or_else(|_| env::var("DATABASE_URL"))
        .expect("DATABASE_URL must be set");

    let manager = ConnectionManager::<PgConnection>::new(database_url);

    info!("Connected to the database");

    Pool::builder()
        .build(manager)
        .expect("Failed to create pool")
});

pub static MESSAGE_QUEUE: LazyLock<Mutex<VecDeque<DbMessage>>> =
    LazyLock::new(|| Mutex::new(VecDeque::new()));

pub fn start_processing_queue() -> tokio::task::JoinHandle<()> {
    return tokio::spawn(async move {
        info!("Started processing thread");

        let mut conn = DB_POOL.get().expect("cant connect");

        loop {
            let maybe_item = {
                let mut queue = MESSAGE_QUEUE.lock().await;
                queue.pop_front()
            };

            if let Some(item) = maybe_item {
                let e = diesel::insert_into(messages)
                    .values(item)
                    .execute(&mut conn);

                if e.is_err() {
                    warn!("{:?}", e);
                }
            } else {
                thread::sleep(Duration::from_millis(1000));
            }
        }
    });
}
