use gtk::prelude::*;

pub async fn run_set_directory_dialog(widget: gtk::Entry) {
    let dialog = gtk::FileChooserDialog::with_buttons::<gtk::ApplicationWindow>(
        None,
        None,
        gtk::FileChooserAction::CreateFolder,
        &[("_Cancel", gtk::ResponseType::Cancel), ("_Select", gtk::ResponseType::Accept)],
    );
    let current_choice = widget.get_text();
    dialog.set_filename(current_choice.as_str());
    let result = woab::run_dialog(&dialog, false).await;
    let filename = dialog.get_filename();
    dialog.close();
    if let (gtk::ResponseType::Accept, Some(filename)) = (result, filename) {
        widget.set_text(&filename.to_string_lossy());
    }
}
