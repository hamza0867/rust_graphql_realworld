use crate::article::resolvers::ArticleMutation;
use crate::db::DbPool;
use crate::user::resolvers::{UsersQuery, UsersMutation};
use juniper::{EmptySubscription, RootNode};
pub struct Context {
    pub db_pool: DbPool,
    pub token: Option<String>,
}

impl juniper::Context for Context {}

pub struct QueryRoot;

#[juniper::graphql_object(Context = Context)]
impl QueryRoot {
    fn users() -> UsersQuery {
        UsersQuery {}
    }
}

pub struct MutationRoot;

#[juniper::graphql_object(Context = Context)]
impl MutationRoot {
    fn users() -> UsersMutation {
        UsersMutation {}
    }

    fn articles() -> ArticleMutation {
        ArticleMutation {}
    }
}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot, EmptySubscription<Context>>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot {}, MutationRoot {}, EmptySubscription::new())
}

