use actix_web::{App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
    })
        .bind("0.0.0.0:9999")?
        .run()
        .await
}
