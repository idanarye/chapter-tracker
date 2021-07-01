use actix::prelude::*;
use gio::prelude::*;

mod msgs;
mod main_app;
mod series;
mod directory;
mod media_types;
mod links_dir;

#[derive(structopt::StructOpt, Debug)]
struct CliArgs {
    #[structopt(long)]
    linksdir: Option<String>,
}

pub fn start_gui() -> anyhow::Result<i32> {
    use structopt::StructOpt;
    let cli_args = CliArgs::from_args();

    gtk::init()?;
    woab::run_actix_inside_gtk_event_loop()?;

    let factories = Factories::new(FactoriesInner::read(&*crate::Asset::get("gui.glade").unwrap())?);

    let app = gtk::Application::new(None, gio::ApplicationFlags::HANDLES_COMMAND_LINE);
    woab::block_on(async {
        let bld = factories.app_main.instantiate();
        let main_app = main_app::MainAppActor::builder()
            .widgets(bld.widgets().unwrap())
            .factories(factories)
            .build()
            .start();
        // app.add_main_option(
            // "linksdir",
            // b'd'.into(),
            // glib::OptionFlags::empty(),
            // glib::OptionArg::FilenameArray,
            // "Maintain directories with symlinks to the unread files",
            // None,
        // );
        // app.connect_handle_local_options(|app, options| {
            // println!("Got {:?}", options.lookup_value("linksdir", None));
            // app.activate();
            // 0
        // });
        app.connect_command_line(|app, _| {
            app.activate();
            0
        });
        woab::route_signal(&app, "activate", "app_activate", main_app.clone()).unwrap();
        woab::route_signal(&app, "shutdown", "app_shutdown", main_app.clone()).unwrap();

        if let Some(links_directory) = cli_args.linksdir {
            main_app.do_send(msgs::MaintainLinksDirectory(links_directory));
        }

        bld.connect_to(main_app);
    });

    let exit_status = app.run();
    Ok(exit_status)
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
