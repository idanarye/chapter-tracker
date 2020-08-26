mod msgs;
mod main_app;
mod series;

pub fn start_gui() -> anyhow::Result<()> {
    gtk::init()?;
    woab::run_actix_inside_gtk_event_loop("chapter-tracker")?;

    let factories = Factories::new(FactoriesInner::read(&*crate::Asset::get("gui.glade").unwrap())?);

    let main_app = factories.clone().app_main.create(|_, widgets| main_app::MainAppActor {
        widgets,
        factories,
    })?;

    main_app.do_send(msgs::UpdateMediaTypesList);
    main_app.do_send(msgs::UpdateSeriesesList);

    gtk::main();
    Ok(())
}

#[derive(woab::Factories)]
pub struct FactoriesInner {
    #[factory(extra(lsm_media_types))]
    pub app_main: woab::ActorFactory<main_app::MainAppActor>,
    pub row_series: woab::ActorFactory<series::SeriesActor>,
}

type Factories = std::rc::Rc<FactoriesInner>;
