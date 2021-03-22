#[macro_use]
extern crate diesel;

pub mod api;
pub mod db;
pub mod handlers;

use db::db_pool::{init_pool, PgPool};

pub fn establish_connection() -> PgPool {
    use dotenv::dotenv;
    use std::env;

    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    init_pool(&database_url).expect("Failed to create pool")
}

#[cfg(test)]
mod tests {
    use super::db::{models::user::*, schema::userinfo::dsl::*};
    use diesel::QueryDsl;
    use diesel::RunQueryDsl;

    #[test]
    fn test_establish_connection() {
        super::establish_connection();
    }

    #[test]
    fn query_user() {
        let pool = super::establish_connection();
        let results = userinfo
            .limit(5)
            .load::<User>(&pool.get().expect("db pool error occured"))
            .expect("Error loading posts");
    }
}
