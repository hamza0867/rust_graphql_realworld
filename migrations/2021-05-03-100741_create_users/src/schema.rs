table! {
    users (id) {
        id -> Int4,
        email -> Varchar,
        username -> Varchar,
        bio -> Nullable<Text>,
        image -> Nullable<Varchar>,
        password_hash -> Nullable<Varchar>,
    }
}
