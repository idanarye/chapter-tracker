use gtk4::prelude::*;

pub async fn run_set_directory_dialog(widget: gtk4::Entry, base_dir: Option<String>) {
    let dialog = gtk4::FileDialog::new();
    let current_choice = widget.text();
    if current_choice.is_empty() {
        if let Some(base_dir) = base_dir {
            dialog.set_initial_folder(Some(&gio::File::for_path(base_dir)));
        }
    } else {
        dialog.set_initial_file(Some(&gio::File::for_path(current_choice)));
    }
    let result = dialog
        .select_folder_future(find_window_widget(widget.clone()).as_ref())
        .await;
    if let Some(filename) = result.ok().and_then(|f| f.path()) {
        widget.set_text(&filename.to_string_lossy());
    }
}

pub fn find_window_widget(widget: impl IsA<gtk4::Widget>) -> Option<gtk4::Window> {
    std::iter::successors(Some(widget.upcast()), |widget| widget.parent())
        .find_map(|widget| widget.downcast::<gtk4::Window>().ok())
}
