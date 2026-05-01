CREATE TABLE messages (
    id VARCHAR PRIMARY KEY,

    author_id VARCHAR,
    author_display_name VARCHAR,
    author_user_name VARCHAR,
    author_server_name VARCHAR,

    author_profile_pic_cdn_id VARCHAR,
    author_clan_tag VARCHAR,

    author_discriminator VARCHAR,
    author_is_bot BOOLEAN,

    channel_id VARCHAR,
    channel_name VARCHAR,

    guild_id VARCHAR,
    guild_name VARCHAR,

    content VARCHAR,
    special_content VARCHAR,

    replied_message_id VARCHAR,

    replied_author_id VARCHAR,
    replied_author_display_name VARCHAR,
    replied_author_user_name VARCHAR,
    replied_author_discriminator VARCHAR,
    replied_author_is_bot BOOLEAN,

    replied_content VARCHAR,

    embeds VARCHAR,
    attachments VARCHAR,

    _timestamp VARCHAR,

    type_ SMALLINT CHECK (type_ >= 0 AND type_ <= 255)
);
