use juniper::GraphQLObject;

#[derive(GraphQLObject)]
#[graphql(description = "A user of the app")]
pub struct User {
    pub email: String,
    pub username: String,
    pub bio: Option<String>,
    pub image: Option<String>,
    pub token: String,
}

#[derive(GraphQLObject)]
#[graphql(description = "The profile of a user")]
pub struct Profile {
    pub username: String,
    pub bio: Option<String>,
    pub image: Option<String>,
    pub following: bool,
}


