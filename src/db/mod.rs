mod pg;

pub use pg::create_weather_table;
pub use pg::insert_weather_data_to_table;
pub use pg::table_exist;

use crate::utils::load_config;
use once_cell::sync::Lazy;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

pub static DB_POOL: Lazy<PgPool> = Lazy::new(|| {
    let config = load_config().unwrap();
    let url = format!(
        "postgre://{}:{}@{}:{}/{}",
        config.database.user,
        config.database.password,
        config.database.host,
        config.database.port,
        config.database.db_name
    );
    PgPoolOptions::new()
        .max_connections(20)
        .connect_lazy(&url)
        .expect("db pool create failed")
});
