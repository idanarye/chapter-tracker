use actix::prelude::*;
use gtk::prelude::*;

use crate::models;
use crate::util::db;

#[derive(typed_builder::TypedBuilder, woab::Removable)]
#[removable(self.widgets.row_directory)]
pub struct DirectoryActor {
    widgets: DirectoryWidgets,
    directory: models::Directory,
}

#[derive(woab::WidgetsFromBuilder, woab::PropSync)]
pub struct DirectoryWidgets {
    pub row_directory: gtk::ListBoxRow,
    #[prop_sync(set, get)]
    txt_directory_pattern: gtk::Entry,
    #[prop_sync(set, get)]
    txt_directory_dir: gtk::Entry,
    #[prop_sync(set, get)]
    txt_directory_volume: gtk::Entry,
    #[prop_sync("active": bool, set, get)]
    chk_directory_recursive: gtk::ToggleButton,
    stk_directory_buttons: gtk::Stack,
    btn_save_directory: gtk::Button,
    btn_cancel_directory_edit: gtk::Button,
}

impl actix::Actor for DirectoryActor {
    type Context = actix::Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        self.update_widgets_from_data();
    }
}

impl DirectoryActor {
    fn update_widgets_from_data(&self) {
        self.widgets.set_props(&DirectoryWidgetsPropSetter {
            txt_directory_pattern: &self.directory.pattern,
            txt_directory_dir: &self.directory.dir,
            txt_directory_volume: &self.directory.volume.map(|v| v.to_string()).unwrap_or("".to_owned()),
            chk_directory_recursive: self.directory.recursive,
        });
    }
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
            "edit_directory" => {
                ctx.spawn(
                    crate::util::edit_mode::EditMode::builder()
                    .stack(self.widgets.stk_directory_buttons.clone())
                    .save_button(self.widgets.btn_save_directory.clone())
                    .cancel_button(self.widgets.btn_cancel_directory_edit.clone())
                    .build()
                    // .with_edit_widget(self
                    .with_edit_widget(self.widgets.txt_directory_pattern.clone(), "changed", self.directory.pattern.clone(), |_| Ok(()))
                    .with_edit_widget(self.widgets.txt_directory_dir.clone(), "changed", self.directory.dir.clone(), |_| Ok(()))
                    .with_edit_widget(self.widgets.txt_directory_volume.clone(), "changed", self.directory.volume.map(|s| s.to_string()).unwrap_or_else(|| "".to_owned()), |text| {
                        if text == "" {
                            return Ok(())
                        }
                        match text.parse::<i64>() {
                            Ok(_) => Ok(()),
                            Err(err) => Err(err.to_string()),
                        }
                    })
                    .with_edit_widget(self.widgets.chk_directory_recursive.clone(), "toggled", self.directory.recursive, |_| Ok(()))
                    .edit_mode(ctx.address().recipient(), ())
                    .into_actor(self)
                    .then(move |user_saved, actor, _| {
                        let directory_id = actor.directory.id;
                        async move {
                            if user_saved {
                                let query = sqlx::query_as("SELECT * FROM directories WHERE id = ?").bind(directory_id);
                                let mut con = db::request_connection().await.unwrap();
                                Some(query.fetch_one(&mut con).await.unwrap())
                            } else {
                                None
                            }
                        }.into_actor(actor)
                    })
                    .then(|result, actor, _| {
                        if let Some(result) = result {
                            actor.directory = result;
                            actor.update_widgets_from_data();
                        }
                        futures::future::ready(())
                    })
                );
                None
            }
            "delete_directory" => {
                let dialog = gtk::MessageDialog::new::<gtk::ApplicationWindow>(
                    None,
                    gtk::DialogFlags::MODAL,
                    gtk::MessageType::Warning,
                    gtk::ButtonsType::YesNo,
                    &format!("Are you sure you want to delete {:?} on {:?}?", self.directory.pattern, self.directory.dir),
                );
                let directory_id = self.directory.id;
                let addr = ctx.address();
                ctx.spawn(async move {
                    let result = woab::run_dialog(
                        &dialog,
                        true,
                    ).await;
                    if result != gtk::ResponseType::Yes {
                        return;
                    }
                    let query = sqlx::query(r#"
                            DELETE FROM directories WHERE id = ?;
                        "#).bind(directory_id);
                    {
                        let mut con = db::request_connection().await.unwrap();
                        query.execute(&mut con).await.unwrap();
                    }
                    addr.send(woab::Remove).await.unwrap();
                }.into_actor(self));
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
        self.directory = data;
        self.update_widgets_from_data();
    }
}

impl actix::Handler<crate::util::edit_mode::InitiateSave> for DirectoryActor {
    type Result = actix::ResponseActFuture<Self, anyhow::Result<()>>;

    fn handle(&mut self, _msg: crate::util::edit_mode::InitiateSave, _ctx: &mut Self::Context) -> Self::Result {
        let directory_id = self.directory.id;
        let DirectoryWidgetsPropGetter {
            txt_directory_pattern,
            txt_directory_dir,
            txt_directory_volume,
            chk_directory_recursive,
        } = self.widgets.get_props();
        Box::pin(async move {
            let query = sqlx::query(r#"
                UPDATE directories
                SET pattern = ?
                  , dir = ?
                  , volume = ?
                  , recursive = ?
                WHERE id == ?
            "#)
                .bind(txt_directory_pattern)
                .bind(txt_directory_dir)
                .bind(if txt_directory_volume == "" {
                    None
                } else {
                    Some(txt_directory_volume.parse::<i64>()?)
                })
                .bind(chk_directory_recursive)
                .bind(directory_id);
            let mut con = db::request_connection().await.unwrap();
            query.execute(&mut con).await?;
            Ok(())
        }.into_actor(self))
    }
}
