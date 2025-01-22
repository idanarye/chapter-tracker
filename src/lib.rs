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
    pub fn css_provider(filename: &str) -> gtk4::CssProvider {
        let css_provider = gtk4::CssProvider::new();
        css_provider.load_from_string(
            std::str::from_utf8(Self::get(filename).unwrap().data.as_ref()).unwrap(),
        );
        css_provider
    }
}

type SqlitePoolConnection = sqlx::pool::PoolConnection<sqlx::Sqlite>;
type SqliteQueryAs<'q, O> = sqlx::query::QueryAs<
    'q,
    sqlx::sqlite::Sqlite,
    O,
    <sqlx::Sqlite as sqlx::Database>::Arguments<'q>,
>;

#[derive(structopt::StructOpt, Debug)]
pub struct CliArgs {
    #[structopt(long)]
    dbfile: Option<String>,
    #[structopt(long)]
    linksdir: Option<String>,
}
