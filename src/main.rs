use actix_web::{web, App, HttpServer, middleware::Logger};
use std::fmt::Display;
use std::error::Error;
use sea_orm::{Database, DatabaseConnection};
use utils::app_state::AppState;
use migration::{Migrator, MigratorTrait};

mod entity;
mod utils;
mod routes;

#[derive(Debug)]
struct MainError {
    message: String
}

impl Display for MainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error: {}", self.message)
    }
}

impl Error for MainError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        &self.message
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}

#[actix_web::main]
async fn main() -> Result<(), MainError> {
    unsafe {
        if std::env::var_os("RUST_LOG").is_none() {
            std::env::set_var("RUST_LOG", "actix_web=info");
        }
    }

    dotenv::dotenv().ok();
    env_logger::init();

    let port = (*utils::constants::PORT).clone();
    let address = (*utils::constants::ADDRESS).clone();
    let db_url = (*utils::constants::DATABASE_URL).clone();
    
    let db: DatabaseConnection = Database::connect(db_url)
        .await
        .map_err(|err| MainError { message: err.to_string() })?;

    Migrator::up(&db, None)
        .await
        .map_err(|err| MainError { message: err.to_string() })?;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState{ db: db.clone() }))
            .wrap(Logger::default())
            .configure(routes::home_routes::config)
            .configure(routes::user_routes::config)
            .configure(routes::auth_routes::config)
            .configure(routes::post_routes::config)
    })
    .bind((address, port))
    .map_err(|err| MainError { message: err.to_string() })?
    .run()
    .await
    .map_err(|err| MainError { message: err.to_string() })
}
