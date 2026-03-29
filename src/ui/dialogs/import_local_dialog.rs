use std::rc::Rc;
use std::sync::mpsc;

use adw::prelude::*;

use crate::application::ServiceFactory;
use crate::application::use_cases::IngestLocalInput;
use crate::infrastructure::tokio_bridge;
use crate::ui::navigation::PAGE_COURSE_LIST;
use crate::ui::state::SharedState;

pub fn show_import_local_dialog(
    state: SharedState,
    stack: Rc<gtk::Stack>,
    parent_window: Option<&gtk::Window>,
    on_success: Option<Rc<dyn Fn()>>,
) {
    let dialog = adw::Dialog::new();
    dialog.set_title("Import Local Media");
    dialog.set_content_width(420);
    dialog.set_content_height(280);

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 12);
    vbox.set_margin_start(16);
    vbox.set_margin_end(16);
    vbox.set_margin_top(16);
    vbox.set_margin_bottom(16);
    vbox.set_valign(gtk::Align::Start);

    let path_label = gtk::Label::new(Some("Media Directory:"));
    path_label.set_halign(gtk::Align::Start);
    vbox.append(&path_label);

    let path_hbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);

    let path_entry = gtk::Entry::new();
    path_entry.set_placeholder_text(Some("Select a directory containing media files"));
    path_entry.set_hexpand(true);
    path_entry.set_sensitive(false);
    path_hbox.append(&path_entry);

    let browse_btn = gtk::Button::with_label("Browse...");
    path_hbox.append(&browse_btn);
    vbox.append(&path_hbox);

    let name_label = gtk::Label::new(Some("Course Name (optional):"));
    name_label.set_halign(gtk::Align::Start);
    vbox.append(&name_label);

    let name_entry = gtk::Entry::new();
    name_entry.set_placeholder_text(Some("Leave empty to use folder name"));
    vbox.append(&name_entry);

    let status_label = gtk::Label::new(None);
    status_label.set_halign(gtk::Align::Start);
    status_label.set_wrap(true);
    vbox.append(&status_label);

    let spinner = gtk::Spinner::new();
    spinner.set_halign(gtk::Align::Start);
    vbox.append(&spinner);

    let button_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    button_box.set_halign(gtk::Align::End);

    let cancel_btn = gtk::Button::with_label("Cancel");
    let import_btn = gtk::Button::with_label("Import");
    import_btn.add_css_class("suggested-action");

    button_box.append(&cancel_btn);
    button_box.append(&import_btn);
    vbox.append(&button_box);

    dialog.set_child(Some(&vbox));

    let dialog_cancel = dialog.clone();
    cancel_btn.connect_clicked(move |_| {
        dialog_cancel.close();
    });

    let parent_browse = parent_window.cloned();
    let path_entry_browse = path_entry.clone();
    browse_btn.connect_clicked(move |_| {
        let file_dialog = gtk::FileDialog::new();
        file_dialog.set_title("Select Media Directory");

        let path_entry_cl = path_entry_browse.clone();
        file_dialog.select_folder(
            parent_browse.as_ref(),
            None::<&gio::Cancellable>,
            move |result| {
                if let Ok(file) = result
                    && let Some(path) = file.path()
                {
                    path_entry_cl.set_text(&path.to_string_lossy());
                }
            },
        );
    });

    let import_btn_cl = import_btn.clone();
    let cancel_btn_for_import = cancel_btn.clone();
    let name_entry_cl = name_entry.clone();
    let status_cl = status_label.clone();
    let spinner_cl = spinner.clone();
    let dialog_cl = dialog.clone();
    let state_cl = state.clone();
    let stack_cl = stack.clone();
    let on_success_cl = on_success;

    import_btn.connect_clicked(move |_| {
        let dir_path = path_entry.text().to_string();
        if dir_path.trim().is_empty() {
            status_cl.set_text("Please select a directory first.");
            return;
        }

        let course_name_input = name_entry_cl.text().to_string();
        let course_name = if course_name_input.trim().is_empty() {
            None
        } else {
            Some(course_name_input.trim().to_string())
        };

        path_entry.set_sensitive(false);
        browse_btn.set_sensitive(false);
        name_entry_cl.set_sensitive(false);
        import_btn_cl.set_sensitive(false);
        cancel_btn_for_import.set_sensitive(false);
        spinner_cl.start();
        status_cl.set_text("Scanning and importing media...");

        let ctx = state_cl.borrow().backend.clone();
        if let Some(ctx) = ctx {
            let (tx, rx): (mpsc::Sender<(String, bool)>, _) = mpsc::channel();

            let dir_idle = path_entry.clone();
            let browse_idle = browse_btn.clone();
            let name_idle = name_entry_cl.clone();
            let import_idle = import_btn_cl.clone();
            let cancel_idle = cancel_btn_for_import.clone();
            let sp_idle = spinner_cl.clone();
            let status_idle = status_cl.clone();
            let dlg_idle = dialog_cl.clone();
            let sk_idle = stack_cl.clone();

            let on_success_cb = on_success_cl.clone();

            glib::idle_add_local(move || {
                if let Ok((msg, success)) = rx.try_recv() {
                    sp_idle.stop();
                    status_idle.set_text(&msg);
                    if success {
                        crate::ui::toast::Toast::show(&msg);
                        dlg_idle.close();
                        sk_idle.set_visible_child_name(PAGE_COURSE_LIST);
                        if let Some(ref cb) = on_success_cb {
                            cb();
                        }
                    } else {
                        dir_idle.set_sensitive(true);
                        browse_idle.set_sensitive(true);
                        name_idle.set_sensitive(true);
                        import_idle.set_sensitive(true);
                        cancel_idle.set_sensitive(true);
                    }
                    glib::ControlFlow::Break
                } else {
                    glib::ControlFlow::Continue
                }
            });

            tokio_bridge::spawn(async move {
                let input = IngestLocalInput { root_path: dir_path, course_name };
                let result = ServiceFactory::ingest_local(&ctx).execute(input).await;
                let (msg, success) = match &result {
                    Ok(output) => (
                        format!(
                            "Imported {} videos across {} modules!",
                            output.videos_count, output.modules_count
                        ),
                        true,
                    ),
                    Err(e) => (format!("Import failed: {e}"), false),
                };
                let _ = tx.send((msg, success));
            });
        } else {
            spinner_cl.stop();
            status_cl.set_text("No backend available.");
            path_entry.set_sensitive(true);
            browse_btn.set_sensitive(true);
            name_entry_cl.set_sensitive(true);
            import_btn_cl.set_sensitive(true);
            cancel_btn_for_import.set_sensitive(true);
        }
    });

    dialog.present(parent_window);
}
