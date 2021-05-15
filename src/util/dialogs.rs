use gtk::prelude::*;

pub async fn run_set_directory_dialog(widget: gtk::Entry, base_dir: Option<String>) {
    let dialog = gtk::FileChooserDialog::with_buttons::<gtk::ApplicationWindow>(
        None,
        None,
        gtk::FileChooserAction::CreateFolder,
        &[("_Cancel", gtk::ResponseType::Cancel), ("_Select", gtk::ResponseType::Accept)],
    );
    let current_choice = widget.get_text();
    if current_choice.is_empty() {
        if let Some(base_dir) = base_dir {
            dialog.set_current_folder(base_dir.as_str());
        }
    } else {
        dialog.set_filename(current_choice.as_str());
    }
    let result = woab::run_dialog(&dialog, false).await;
    let filename = dialog.get_filename();
    dialog.close();
    if let (gtk::ResponseType::Accept, Some(filename)) = (result, filename) {
        widget.set_text(&filename.to_string_lossy());
    }
}
