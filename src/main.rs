use actix_web::{App, HttpServer};
use sitevik::{Config, static_files};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = Config::from_env()
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidInput, error))?;
    let root = config.root;
    let spa = config.spa;

    HttpServer::new(move || App::new().service(static_files(root.clone(), spa)))
        .bind(config.bind_addr)?
        .run()
        .await
}
