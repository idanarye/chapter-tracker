use gtk4::prelude::*;

pub async fn run_set_directory_dialog(widget: gtk4::Entry, base_dir: Option<String>) {
    let dialog = gtk4::FileChooserDialog::new(
        None::<&str>,
        find_window_widget(widget.clone()).as_ref(),
        gtk4::FileChooserAction::SelectFolder,
        &[
            ("_Cancel", gtk4::ResponseType::Cancel),
            ("_Select", gtk4::ResponseType::Accept),
        ],
    );

    let current_choice = widget.text();
    if current_choice.is_empty() {
        if let Some(base_dir) = base_dir {
            dialog
                .set_current_folder(Some(&gio::File::for_path(base_dir)))
                .unwrap();
        }
    } else {
        dialog
            .set_file(&gio::File::for_path(current_choice))
            .unwrap();
    }
    let result = dialog.run_future().await;
    let filename = dialog.file().and_then(|f| f.path());
    dialog.close();
    if let (gtk4::ResponseType::Accept, Some(filename)) = (result, filename) {
        widget.set_text(&filename.to_string_lossy());
    }
}

pub fn find_window_widget(widget: impl IsA<gtk4::Widget>) -> Option<gtk4::Window> {
    std::iter::successors(Some(widget.upcast()), |widget| widget.parent())
        .find_map(|widget| widget.downcast::<gtk4::Window>().ok())
}
