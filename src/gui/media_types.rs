use actix::prelude::*;
use gtk::prelude::*;

use crate::models;

#[derive(typed_builder::TypedBuilder)]
pub struct MediaTypesActor {
    factories: crate::gui::Factories,
    widgets: MediaTypesWindowWidgets,
    #[builder(setter(skip), default)]
    media_types: hashbrown::HashMap<i64, MediaTypeRow>,
}

#[derive(woab::WidgetsFromBuilder)]
pub struct MediaTypesWindowWidgets {
    win_media_types: gtk::Window,
    lst_media_types: gtk::ListBox,
}

impl actix::Actor for MediaTypesActor {
    type Context = actix::Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.widgets.win_media_types.show();

        ctx.spawn(
            crate::actors::DbActor::from_registry().send(crate::msgs::RefreshList {
                orig_ids: Default::default(),
                query: sqlx::query_as("SELECT * FROM media_types"),
                id_dlg: |row_data: &models::MediaType| row_data.id,
                addr: ctx.address(),
            })
            .into_actor(self)
            .then(move |_, _, _| {
                futures::future::ready(())
            })
        );
    }
}

impl actix::Handler<woab::Signal> for MediaTypesActor {
    type Result = woab::SignalResult;

    fn handle(&mut self, msg: woab::Signal, _ctx: &mut Self::Context) -> Self::Result {
        Ok(match msg.name() {
            _ => msg.cant_handle()?,
        })
    }
}

impl actix::Handler<crate::msgs::UpdateListRowData<models::MediaType>> for MediaTypesActor {
    type Result = ();

    fn handle(&mut self, msg: crate::msgs::UpdateListRowData<models::MediaType>, ctx: &mut Self::Context) -> Self::Result {
        let crate::msgs::UpdateListRowData(data) = msg;
        match self.media_types.entry(data.id) {
            hashbrown::hash_map::Entry::Occupied(mut entry) => {
                let entry = entry.get_mut();
                if entry.model != data {
                    entry.model = data;
                    entry.update_widgets_from_model();
                    entry.widgets.row_media_type.changed();
                }
            }
            hashbrown::hash_map::Entry::Vacant(entry) => {
                let widgets: MediaTypeWidgets = self.factories.row_media_type.instantiate().connect_to((data.id, ctx.address())).widgets().unwrap();
                let entry = entry.insert(MediaTypeRow {model: data, widgets});
                entry.update_widgets_from_model();
                self.widgets.lst_media_types.add(&entry.widgets.row_media_type);
            }
        }
    }
}

struct MediaTypeRow {
    model: models::MediaType,
    widgets: MediaTypeWidgets,
}

#[derive(woab::WidgetsFromBuilder, woab::PropSync)]
struct MediaTypeWidgets {
    row_media_type: gtk::ListBoxRow,
    #[prop_sync(set)]
    txt_media_type_name: gtk::Entry,
    #[prop_sync(set)]
    txt_media_type_base_dir: gtk::Entry,
    #[prop_sync(set)]
    txt_media_type_file_types: gtk::Entry,
    #[prop_sync(set)]
    txt_media_type_program: gtk::Entry,
}

impl MediaTypeRow {
    fn update_widgets_from_model(&self) {
        self.widgets.set_props(&MediaTypeWidgetsPropSetter {
            txt_media_type_name: &self.model.name,
            txt_media_type_base_dir: &self.model.base_dir,
            txt_media_type_file_types: &self.model.file_types,
            txt_media_type_program: &self.model.program,
        });
    }
}

impl actix::Handler<woab::Signal<i64>> for MediaTypesActor {
    type Result = woab::SignalResult;

    fn handle(&mut self, msg: woab::Signal<i64>, _ctx: &mut Self::Context) -> Self::Result {
        Ok(match msg.name() {
            _ => msg.cant_handle()?,
        })
    }
}
