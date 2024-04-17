use actix::prelude::*;
use gio::prelude::*;

mod directory;
mod links_dir;
mod main_app;
mod media_types;
mod msgs;
mod series;

pub fn start_gui() -> woab::Result<()> {
    let app = gtk4::Application::builder()
        .flags(gio::ApplicationFlags::HANDLES_COMMAND_LINE)
        .build();
    woab::main(app, |app| {
        use structopt::StructOpt;
        let cli_args = crate::CliArgs::from_args();

        let factories = Factories::new(FactoriesInner::read(
            &*crate::Asset::get("gui.glade").unwrap().data,
        )?);

        let ctx = Context::new();
        woab::route_signal(app, "activate", "app_activate", ctx.address())?;
        woab::route_signal(app, "shutdown", "app_shutdown", ctx.address())?;
        if let Some(links_directory) = cli_args.linksdir {
            ctx.address()
                .do_send(msgs::MaintainLinksDirectory(links_directory));
        }
        let bld = factories.app_main.instantiate_route_to(ctx.address());
        ctx.run(
            main_app::MainAppActor::builder()
                .widgets(bld.widgets().unwrap())
                .factories(factories)
                .build(),
        );
        Ok(())
    })
}

#[derive(woab::Factories)]
pub struct FactoriesInner {
    #[factory(extra(lsm_media_types))]
    pub app_main: woab::BuilderFactory,
    pub row_series: woab::BuilderFactory,
    pub row_episode: woab::BuilderFactory,
    #[factory(extra(lsm_directory_scan_preview, srt_directory_scan_preview))]
    pub row_directory: woab::BuilderFactory,

    pub win_media_types: woab::BuilderFactory,
    pub row_media_type: woab::BuilderFactory,
}

type Factories = std::rc::Rc<FactoriesInner>;
