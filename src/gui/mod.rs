use actix::prelude::*;
use gtk4::prelude::*;

use crate::gui::gobjects::MediaTypeGObject;

mod directory;
mod gobjects;
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

        app.connect_command_line(|app, _| {
            app.activate();
            0
        });

        let factories = Factories::new(FactoriesInner {
            main: FactoriesMain::read(&*crate::Asset::get("main.ui").unwrap().data)?,
            media_types: FactoriesMediaTypes::read(
                &*crate::Asset::get("media_types.ui").unwrap().data,
            )?,
        });

        let ctx = Context::new();
        woab::route_signal(app, "activate", "app_activate", ctx.address())?;
        woab::route_signal(app, "shutdown", "app_shutdown", ctx.address())?;
        if let Some(links_directory) = cli_args.linksdir {
            ctx.address()
                .do_send(msgs::MaintainLinksDirectory(links_directory));
        }
        let bld = factories.main.app_main.instantiate_route_to(ctx.address());

        ctx.run(
            main_app::MainAppActor::builder()
                .widgets(bld.widgets().unwrap())
                .factories(factories)
                .lsm_media_types(gio::ListStore::new::<MediaTypeGObject>())
                .build(),
        );
        Ok(())
    })
}

#[derive(woab::Factories)]
pub struct FactoriesMain {
    pub app_main: woab::BuilderFactory,
    pub row_series: woab::BuilderFactory,
    pub row_episode: woab::BuilderFactory,
    //#[factory(extra(lsm_directory_scan_preview, srt_directory_scan_preview))]
    pub row_directory: woab::BuilderFactory,
}

#[derive(woab::Factories)]
pub struct FactoriesMediaTypes {
    pub win_media_types: woab::BuilderFactory,
    pub row_media_type: woab::BuilderFactory,
}

pub struct FactoriesInner {
    main: FactoriesMain,
    media_types: FactoriesMediaTypes,
}

type Factories = std::rc::Rc<FactoriesInner>;
