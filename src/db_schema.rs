table! {
    articles (id) {
        id -> Int4,
        slug -> Varchar,
        title -> Varchar,
        description -> Nullable<Varchar>,
        body -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        author_id -> Int4,
    }
}

table! {
    follows (follower_id, followed_id) {
        follower_id -> Int4,
        followed_id -> Int4,
        active -> Bool,
    }
}

table! {
    tag_article (tag, article_id) {
        tag -> Varchar,
        article_id -> Int4,
    }
}

table! {
    tags (tag) {
        tag -> Varchar,
    }
}

table! {
    user_favorites_article (user_id, article_id) {
        user_id -> Int4,
        article_id -> Int4,
        active -> Bool,
    }
}

table! {
    users (id) {
        id -> Int4,
        email -> Varchar,
        username -> Varchar,
        bio -> Nullable<Text>,
        image -> Nullable<Varchar>,
        password_hash -> Varchar,
    }
}

joinable!(articles -> users (author_id));
joinable!(tag_article -> articles (article_id));
joinable!(tag_article -> tags (tag));
joinable!(user_favorites_article -> articles (article_id));
joinable!(user_favorites_article -> users (user_id));

allow_tables_to_appear_in_same_query!(
    articles,
    follows,
    tag_article,
    tags,
    user_favorites_article,
    users,
);
