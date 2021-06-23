use juniper::GraphQLObject;
use chrono::{DateTime, Utc};

#[derive(GraphQLObject)]
#[graphql(description = "An aticle")]
pub struct Article {
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub author: crate::user::model::Profile,
    pub tag_list: Vec<String>,
    pub favorited: bool,
    pub favorites_count: i32,
}



