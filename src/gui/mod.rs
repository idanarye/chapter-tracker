use actix::prelude::*;

mod msgs;
mod main_app;
mod series;
mod directory;
mod media_types;

pub fn start_gui() -> anyhow::Result<()> {
    gtk::init()?;
    woab::run_actix_inside_gtk_event_loop()?;

    let factories = Factories::new(FactoriesInner::read(&*crate::Asset::get("gui.glade").unwrap())?);

    woab::block_on(async {
        factories.app_main.instantiate().connect_with(|bld| {
            let main_app = main_app::MainAppActor::builder()
                .widgets(bld.widgets().unwrap())
                .factories(factories)
                .build()
                .start();
            main_app
        });
    });

    gtk::main();
    Ok(())
}

#[derive(woab::Factories)]
pub struct FactoriesInner {
    #[factory(extra(lsm_media_types))]
    pub app_main: woab::BuilderFactory,
    pub row_series: woab::BuilderFactory,
    pub row_episode: woab::BuilderFactory,
    pub row_directory: woab::BuilderFactory,

    pub win_media_types: woab::BuilderFactory,
    pub row_media_type: woab::BuilderFactory,
}

type Factories = std::rc::Rc<FactoriesInner>;
