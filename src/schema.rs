// @generated automatically by Diesel CLI.

diesel::table! {
    images (id) {
        id -> Varchar,
        content -> Nullable<Varchar>,
        removal_reason -> Nullable<Varchar>,
        original_url -> Nullable<Varchar>,
    }
}

diesel::table! {
    messages (id) {
        id -> Varchar,
        author_id -> Nullable<Varchar>,
        author_display_name -> Nullable<Varchar>,
        author_user_name -> Nullable<Varchar>,
        author_server_name -> Nullable<Varchar>,
        author_profile_pic_cdn_id -> Nullable<Varchar>,
        author_clan_tag -> Nullable<Varchar>,
        author_discriminator -> Nullable<Varchar>,
        author_is_bot -> Nullable<Bool>,
        channel_id -> Nullable<Varchar>,
        channel_name -> Nullable<Varchar>,
        guild_id -> Nullable<Varchar>,
        guild_name -> Nullable<Varchar>,
        content -> Nullable<Varchar>,
        special_content -> Nullable<Varchar>,
        replied_message_id -> Nullable<Varchar>,
        replied_author_id -> Nullable<Varchar>,
        replied_author_display_name -> Nullable<Varchar>,
        replied_author_user_name -> Nullable<Varchar>,
        replied_author_discriminator -> Nullable<Varchar>,
        replied_author_is_bot -> Nullable<Bool>,
        replied_content -> Nullable<Varchar>,
        embeds -> Nullable<Varchar>,
        attachments -> Nullable<Varchar>,
        _timestamp -> Nullable<Varchar>,
        type_ -> Nullable<Int2>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    images,
    messages,
);
