#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

pub mod schema;
pub mod models;

pub mod scan;

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

pub fn establish_connection(database: &str) -> SqliteConnection {
    SqliteConnection::establish(database).unwrap()
}

#[allow(unused_imports, dead_code)]
mod migrations {
    use diesel::sqlite::SqliteConnection;
    use diesel_migrations;

    embed_migrations!();


    pub fn run_migrations(con: &SqliteConnection) -> Result<(), diesel_migrations::RunMigrationsError> {
        embedded_migrations::run(con)
    }
}

pub use migrations::run_migrations;
