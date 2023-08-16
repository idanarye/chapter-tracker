pub mod actors;
pub mod files_discovery;
mod gui;
pub mod links_handling;
mod models;
pub mod msgs;
mod util;

pub use gui::start_gui;

#[derive(rust_embed::RustEmbed)]
#[folder = "assets"]
struct Asset;

impl Asset {
    pub fn css_provider(filename: &str) -> gtk::CssProvider {
        use gtk::prelude::*;
        let css_provider = gtk::CssProvider::new();
        css_provider
            .load_from_data(Self::get(filename).unwrap().as_ref())
            .unwrap();
        css_provider
    }
}

type SqlitePoolConnection = sqlx::pool::PoolConnection<sqlx::Sqlite>;
type SqliteQueryAs<'q, O> = sqlx::query::QueryAs<
    'q,
    sqlx::sqlite::Sqlite,
    O,
    <sqlx::sqlite::Sqlite as sqlx::database::HasArguments<'q>>::Arguments,
>;

#[derive(structopt::StructOpt, Debug)]
pub struct CliArgs {
    #[structopt(long)]
    dbfile: Option<String>,
    #[structopt(long)]
    linksdir: Option<String>,
}
