use actix::prelude::*;
use gtk::prelude::*;

use crate::models;

#[derive(typed_builder::TypedBuilder, woab::Removable)]
#[removable(self.widgets.row_directory)]
pub struct DirectoryActor {
    widgets: DirectoryWidgets,
}

#[derive(woab::WidgetsFromBuilder, woab::PropSync)]
pub struct DirectoryWidgets {
    pub row_directory: gtk::ListBoxRow,
    #[prop_sync(set)]
    txt_directory_pattern: gtk::Entry,
    #[prop_sync(set)]
    txt_directory_dir: gtk::Entry,
    #[prop_sync(set)]
    txt_directory_volume: gtk::Entry,
    #[prop_sync("active": bool, set)]
    chk_directory_recursive: gtk::ToggleButton,
}

impl actix::Actor for DirectoryActor {
    type Context = actix::Context<Self>;
}

impl actix::Handler<woab::Signal> for DirectoryActor {
    type Result = woab::SignalResult;

    fn handle(&mut self, msg: woab::Signal, ctx: &mut Self::Context) -> Self::Result {
        Ok(match msg.name() {
            "open_directory_dir_dialog" => {
                if self.widgets.txt_directory_dir.get_editable() {
                    ctx.spawn(crate::util::dialogs::run_set_directory_dialog(self.widgets.txt_directory_dir.clone()).into_actor(self));
                }
                None
            }
            _ => msg.cant_handle()?,
        })
    }
}

impl actix::Handler<crate::msgs::UpdateListRowData<models::Directory>> for DirectoryActor {
    type Result = ();

    fn handle(&mut self, msg: crate::msgs::UpdateListRowData<models::Directory>, _ctx: &mut Self::Context) -> Self::Result {
        let crate::msgs::UpdateListRowData(data) = msg;
        self.widgets.set_props(&DirectoryWidgetsPropSetter {
            txt_directory_pattern: &data.pattern,
            txt_directory_dir: &data.dir,
            txt_directory_volume: &data.volume.map(|v| v.to_string()).unwrap_or("".to_owned()),
            chk_directory_recursive: data.recursive,
        });
    }
}
