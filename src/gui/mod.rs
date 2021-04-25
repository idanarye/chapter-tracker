use actix::prelude::*;

mod msgs;
mod main_app;
mod series;

pub fn start_gui() -> anyhow::Result<()> {
    gtk::init()?;
    woab::run_actix_inside_gtk_event_loop()?;

    let factories = Factories::new(FactoriesInner::read(&*crate::Asset::get("gui.glade").unwrap())?);

    woab::block_on(async {
        factories.app_main.instantiate().connect_with(|bld| {
            let main_app = main_app::MainAppActor {
                widgets: bld.widgets().unwrap(),
                factories,
            }.start();
            main_app.do_send(msgs::UpdateMediaTypesList);
            main_app.do_send(msgs::UpdateSeriesesList);
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
}

type Factories = std::rc::Rc<FactoriesInner>;