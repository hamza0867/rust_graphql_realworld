use diesel::r2d2::{Pool, ConnectionManager};
use diesel::pg::PgConnection;
use dotenv::dotenv;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub fn get_db_pool() -> DbPool {
    dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(db_url);
    Pool::new(manager).expect("Failed to create DB Pool")
}
