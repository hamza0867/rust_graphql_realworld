use juniper::{EmptySubscription, RootNode};

use crate::db::DbPool;

pub struct Context {
    pub db_pool: DbPool,
    pub token: Option<String>
}

impl juniper::Context for Context {}

pub struct QueryRoot;

pub struct MutationRoot;

pub type Schema = RootNode<'static, QueryRoot, MutationRoot, EmptySubscription<Context>>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot {}, MutationRoot {}, EmptySubscription::new())
}

