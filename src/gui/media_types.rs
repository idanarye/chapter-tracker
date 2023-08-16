use actix::prelude::*;
use gtk::prelude::*;

use crate::models;
use crate::util::db;
use crate::util::edit_mode::EditMode;

#[derive(typed_builder::TypedBuilder)]
pub struct MediaTypesActor {
    factories: crate::gui::Factories,
    widgets: MediaTypesWindowWidgets,
    main_app: actix::Addr<crate::gui::main_app::MainAppActor>,
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
            crate::actors::DbActor::from_registry()
                .send(crate::msgs::RefreshList {
                    orig_ids: Default::default(),
                    query: sqlx::query_as("SELECT * FROM media_types"),
                    id_dlg: |row_data: &models::MediaType| row_data.id,
                    addr: ctx.address(),
                })
                .into_actor(self)
                .then(move |_, actor, ctx| {
                    actor.add_new_entry_row(ctx);
                    futures::future::ready(())
                }),
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

    fn handle(
        &mut self,
        msg: crate::msgs::UpdateListRowData<models::MediaType>,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        for data in msg.0 {
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
                    let widgets: MediaTypeWidgets = self
                        .factories
                        .row_media_type
                        .instantiate()
                        .connect_to((data.id, ctx.address()))
                        .widgets()
                        .unwrap();
                    let entry = entry.insert(MediaTypeRow {
                        model: data,
                        widgets,
                    });
                    entry.update_widgets_from_model();
                    self.widgets
                        .lst_media_types
                        .add(&entry.widgets.row_media_type);
                }
            }
        }
    }
}

impl MediaTypesActor {
    fn add_new_entry_row(&mut self, ctx: &mut actix::Context<Self>) {
        let data = models::MediaType {
            id: -1,
            name: "".to_owned(),
            base_dir: "".to_owned(),
            file_types: "".to_owned(),
            adjacent_file_types: "".to_owned(),
            program: "".to_owned(),
            maintain_symlinks: false,
        };
        let entry =
            if let hashbrown::hash_map::Entry::Vacant(entry) = self.media_types.entry(data.id) {
                entry
            } else {
                return;
            };
        let widgets: MediaTypeWidgets = self
            .factories
            .row_media_type
            .instantiate()
            .connect_to((data.id, ctx.address()))
            .widgets()
            .unwrap();
        let entry = entry.insert(MediaTypeRow {
            model: data,
            widgets,
        });
        entry.update_widgets_from_model();
        self.widgets
            .lst_media_types
            .add(&entry.widgets.row_media_type);
        ctx.spawn(
            entry
                .add_verifications_to_edit_mode(
                    EditMode::builder()
                        .stack(entry.widgets.stk_media_type_edit.clone())
                        .stack_page("new")
                        .save_button(entry.widgets.btn_save_new_media_type.clone())
                        .build(),
                )
                .edit_mode(ctx.address().recipient(), entry.model.id)
                .into_actor(self)
                .then(move |user_saved, actor, _| {
                    async move {
                        if let Some(media_type_id) = user_saved {
                            let query = sqlx::query_as("SELECT * FROM media_types WHERE id = ?")
                                .bind(media_type_id);
                            let mut con = db::request_connection().await.unwrap();
                            Some(query.fetch_one(&mut con).await.unwrap())
                        } else {
                            None
                        }
                    }
                    .into_actor(actor)
                })
                .then(|result, actor, ctx| {
                    if let Some(result) = result {
                        let mut row = actor
                            .media_types
                            .remove(&-1)
                            .expect("entry with id=-1 should have been in media_types");
                        row.model = result;
                        row.update_widgets_from_model();
                        actor.media_types.insert(row.model.id, row);
                        actor
                            .main_app
                            .do_send(crate::gui::msgs::UpdateMediaTypesList);
                        actor.add_new_entry_row(ctx);
                    }
                    futures::future::ready(())
                }),
        );
    }
}

struct MediaTypeRow {
    model: models::MediaType,
    widgets: MediaTypeWidgets,
}

#[derive(woab::WidgetsFromBuilder, woab::PropSync)]
struct MediaTypeWidgets {
    row_media_type: gtk::ListBoxRow,
    #[prop_sync(set, get)]
    txt_media_type_name: gtk::Entry,
    #[prop_sync(set, get)]
    txt_media_type_base_dir: gtk::Entry,
    #[prop_sync(set, get)]
    txt_media_type_file_types: gtk::Entry,
    #[prop_sync(set, get)]
    txt_media_type_adjacent_file_types: gtk::Entry,
    #[prop_sync(set, get)]
    txt_media_type_program: gtk::Entry,
    #[prop_sync("active": bool, set, get)]
    chk_media_type_maintain_symlinks: gtk::ToggleButton,
    stk_media_type_edit: gtk::Stack,
    btn_save_media_type: gtk::Button,
    btn_cancel_media_type_edit: gtk::Button,
    btn_save_new_media_type: gtk::Button,
}

impl MediaTypeRow {
    fn update_widgets_from_model(&self) {
        self.widgets.set_props(&MediaTypeWidgetsPropSetter {
            txt_media_type_name: &self.model.name,
            txt_media_type_base_dir: &self.model.base_dir,
            txt_media_type_file_types: &self.model.file_types,
            txt_media_type_adjacent_file_types: &self.model.adjacent_file_types,
            txt_media_type_program: &self.model.program,
            chk_media_type_maintain_symlinks: self.model.maintain_symlinks,
        });
    }

    fn add_verifications_to_edit_mode(&self, edit_mode: EditMode) -> EditMode {
        edit_mode
            .with_edit_widget(
                self.widgets.txt_media_type_name.clone(),
                "changed",
                self.model.name.clone(),
                |text| {
                    if text == "" {
                        Err("media type name cannot be empty".to_owned())
                    } else {
                        Ok(())
                    }
                },
            )
            .with_edit_widget(
                self.widgets.txt_media_type_base_dir.clone(),
                "changed",
                self.model.base_dir.clone(),
                |_| Ok(()),
            )
            .with_edit_widget(
                self.widgets.txt_media_type_file_types.clone(),
                "changed",
                self.model.file_types.clone(),
                |_| Ok(()),
            )
            .with_edit_widget(
                self.widgets.txt_media_type_adjacent_file_types.clone(),
                "changed",
                self.model.adjacent_file_types.clone(),
                |_| Ok(()),
            )
            .with_edit_widget(
                self.widgets.txt_media_type_program.clone(),
                "changed",
                self.model.program.clone(),
                |_| Ok(()),
            )
            .with_edit_widget(
                self.widgets.chk_media_type_maintain_symlinks.clone(),
                "toggled",
                self.model.maintain_symlinks.clone(),
                |_| Ok(()),
            )
    }
}

impl actix::Handler<woab::Signal<i64>> for MediaTypesActor {
    type Result = woab::SignalResult;

    fn handle(&mut self, msg: woab::Signal<i64>, ctx: &mut Self::Context) -> Self::Result {
        let media_type_id = *msg.tag();
        let media_type = &self.media_types[&media_type_id];
        Ok(match msg.name() {
            "open_base_directory_dialog" => {
                let icon_position: gtk::EntryIconPosition = msg.param(1)?;
                match (
                    media_type.widgets.txt_media_type_base_dir.is_editable(),
                    icon_position,
                ) {
                    (true, gtk::EntryIconPosition::Primary) => {
                        ctx.spawn(
                            crate::util::dialogs::run_set_directory_dialog(
                                media_type.widgets.txt_media_type_base_dir.clone(),
                                None,
                            )
                            .into_actor(self),
                        );
                    }
                    (true, gtk::EntryIconPosition::Secondary) => {
                        media_type.widgets.txt_media_type_base_dir.set_text("");
                    }
                    _ => (),
                }

                None
            }
            "edit_media_type" => {
                ctx.spawn(
                    media_type
                        .add_verifications_to_edit_mode(
                            EditMode::builder()
                                .stack(media_type.widgets.stk_media_type_edit.clone())
                                .save_button(media_type.widgets.btn_save_media_type.clone())
                                .cancel_button(
                                    media_type.widgets.btn_cancel_media_type_edit.clone(),
                                )
                                .build(),
                        )
                        .edit_mode(ctx.address().recipient(), media_type_id)
                        .into_actor(self)
                        .then(move |_, actor, ctx| {
                            crate::actors::DbActor::from_registry()
                                .send(crate::msgs::RefreshList {
                                    orig_ids: Default::default(),
                                    query: sqlx::query_as("SELECT * FROM media_types WHERE id = ?")
                                        .bind(media_type_id),
                                    id_dlg: |row_data: &models::MediaType| row_data.id,
                                    addr: ctx.address(),
                                })
                                .into_actor(actor)
                                .then(move |_, _, _| futures::future::ready(()))
                        }),
                );
                None
            }
            "delete_media_type" => {
                let media_type_name = media_type.model.name.clone();
                ctx.spawn(
                    async move {
                        let mut con = db::request_connection().await.unwrap();
                        let (num_serieses,): (i64,) =
                            sqlx::query_as("SELECT COUNT(*) FROM serieses WHERE media_type = ?")
                                .bind(media_type_id)
                                .fetch_one(&mut con)
                                .await
                                .unwrap();
                        if 0 < num_serieses {
                            woab::run_dialog(
                                &gtk::MessageDialog::new::<gtk::Window>(
                                    None,
                                    gtk::DialogFlags::MODAL,
                                    gtk::MessageType::Error,
                                    gtk::ButtonsType::Close,
                                    &format!(
                                        "Cannot delete {:?} - {} serieses are using it",
                                        media_type_name, num_serieses
                                    ),
                                ),
                                true,
                            )
                            .await;
                            false
                        } else {
                            let user_decision = woab::run_dialog(
                                &gtk::MessageDialog::new::<gtk::ApplicationWindow>(
                                    None,
                                    gtk::DialogFlags::MODAL,
                                    gtk::MessageType::Warning,
                                    gtk::ButtonsType::YesNo,
                                    &format!(
                                        "Are you sure you want to delete {:?}?",
                                        media_type_name
                                    ),
                                ),
                                true,
                            )
                            .await;
                            if user_decision == gtk::ResponseType::Yes {
                                sqlx::query("DELETE FROM media_types WHERE id = ?")
                                    .bind(media_type_id)
                                    .execute(&mut con)
                                    .await
                                    .unwrap();
                                true
                            } else {
                                false
                            }
                        }
                    }
                    .into_actor(self)
                    .then(move |did_delete, actor, _ctx| {
                        if did_delete {
                            actor
                                .widgets
                                .lst_media_types
                                .remove(&actor.media_types[&media_type_id].widgets.row_media_type);
                            actor
                                .main_app
                                .do_send(crate::gui::msgs::UpdateMediaTypesList);
                        }
                        futures::future::ready(())
                    }),
                );
                None
            }
            _ => msg.cant_handle()?,
        })
    }
}

impl actix::Handler<crate::util::edit_mode::InitiateSave<i64>> for MediaTypesActor {
    type Result = actix::ResponseActFuture<Self, anyhow::Result<i64>>;

    fn handle(
        &mut self,
        msg: crate::util::edit_mode::InitiateSave<i64>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let media_type_id = msg.0;
        let media_type = &self.media_types[&media_type_id];
        let MediaTypeWidgetsPropGetter {
            txt_media_type_name,
            txt_media_type_base_dir,
            txt_media_type_file_types,
            txt_media_type_adjacent_file_types,
            txt_media_type_program,
            chk_media_type_maintain_symlinks,
        } = media_type.widgets.get_props();
        let main_app = self.main_app.clone();
        Box::pin(async move {
            if media_type_id < 0 {
                let query = sqlx::query(r#"
                    INSERT INTO media_types(name, base_dir, file_types, adjacent_file_types, program, maintain_symlinks)
                    VALUES(?, ?, ?, ?, ?, ?)
                "#)
                    .bind(txt_media_type_name)
                    .bind(txt_media_type_base_dir)
                    .bind(txt_media_type_file_types)
                    .bind(txt_media_type_adjacent_file_types)
                    .bind(txt_media_type_program)
                    .bind(chk_media_type_maintain_symlinks);
                let mut con = db::request_connection().await?;
                let query_result = query.execute(&mut con).await?;
                Ok(query_result.last_insert_rowid())
            } else {
                let query = sqlx::query(r#"
                    UPDATE media_types
                    SET name = ?
                      , base_dir = ?
                      , file_types = ?
                      , adjacent_file_types = ?
                      , program = ?
                      , maintain_symlinks = ?
                    WHERE id = ?
                "#)
                    .bind(txt_media_type_name)
                    .bind(txt_media_type_base_dir)
                    .bind(txt_media_type_file_types)
                    .bind(txt_media_type_adjacent_file_types)
                    .bind(txt_media_type_program)
                    .bind(chk_media_type_maintain_symlinks)
                    .bind(media_type_id);
                let mut con = db::request_connection().await?;
                query.execute(&mut con).await?;
                main_app.do_send(crate::gui::msgs::UpdateMediaTypesList);
                Ok(media_type_id)
            }
        }.into_actor(self))
    }
}
