use std::env;

use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{pg::PgConnection, r2d2::PooledConnection};

pub type PgPool = Pool<ConnectionManager<PgConnection>>;
pub type Conn = PooledConnection<ConnectionManager<PgConnection>>;

pub fn create_connection_pool() -> PgPool {
    let db_url = env::var("DATABASE_URL").expect("Can't get DB URL");
    let manager = ConnectionManager::<PgConnection>::new(db_url);
    Pool::builder()
        .build(manager)
        .expect("Failed to create pool")
}
