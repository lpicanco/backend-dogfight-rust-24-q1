use std::env;

use actix_web::{web, App, HttpServer};
use deadpool_postgres::{Config, ManagerConfig, RecyclingMethod, Runtime};
use tokio_postgres::NoTls;

mod account_statement_handler;
mod model;
mod transaction_handler;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let database_url = env::var("DATABASE_URL").unwrap();
    let port = env::var("PORT").unwrap_or("9999".to_string());

    let mut cfg = Config::new();
    cfg.url = Some(database_url);
    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });
    cfg.get_pool_config().max_size = 60;

    let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();

    println!("🦀Server running at http://localhost:{}/", port);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(transaction_handler::handle)
            .service(account_statement_handler::handle)
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}
