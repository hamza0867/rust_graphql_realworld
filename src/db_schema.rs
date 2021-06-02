table! {
    follows (follower_id, followed_id) {
        follower_id -> Int4,
        followed_id -> Int4,
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

allow_tables_to_appear_in_same_query!(
    follows,
    users,
);
