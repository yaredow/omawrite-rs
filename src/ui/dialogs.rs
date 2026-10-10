use crate::core::storage;
use gtk4::prelude::*;
use relm4::adw;
use relm4::adw::prelude::*;
use std::path::PathBuf;

pub fn prompt_discard<FSave, FDiscard>(
    window: &adw::ApplicationWindow,
    on_save: FSave,
    on_discard: FDiscard,
) where
    FSave: Fn() + 'static,
    FDiscard: Fn() + 'static,
{
    let alert = adw::AlertDialog::new(
        Some("Save Changes?"),
        Some("The current document has unsaved modifications. Unsaved work will be lost."),
    );
    alert.add_response("cancel", "Cancel");
    alert.add_response("discard", "Discard");
    alert.add_response("save", "Save");
    alert.set_response_appearance("discard", adw::ResponseAppearance::Destructive);
    alert.set_response_appearance("save", adw::ResponseAppearance::Suggested);
    alert.set_default_response(Some("save"));
    alert.set_close_response("cancel");

    alert.choose(
        window,
        gio::Cancellable::NONE,
        move |response| match response.as_str() {
            "save" => on_save(),
            "discard" => on_discard(),
            _ => {}
        },
    );
}

pub fn prompt_external_modified<FReload, FKeep>(
    window: &adw::ApplicationWindow,
    is_dirty: bool,
    on_reload: FReload,
    on_keep: FKeep,
) where
    FReload: Fn() + 'static,
    FKeep: Fn() + 'static,
{
    let alert = adw::AlertDialog::new(
        Some("File changed"),
        Some(if is_dirty {
            "This file changed outside Omawrite. Reloading will discard your changes."
        } else {
            "This file changed outside Omawrite."
        }),
    );
    alert.add_response("keep", "Keep Mine");
    alert.add_response("reload", "Reload");
    alert.set_response_appearance("reload", adw::ResponseAppearance::Suggested);
    alert.set_default_response(Some("reload"));
    alert.set_close_response("keep");

    alert.choose(
        window,
        gio::Cancellable::NONE,
        move |response| match response.as_str() {
            "reload" => on_reload(),
            _ => on_keep(),
        },
    );
}

pub fn prompt_external_deleted<FKeep>(window: &adw::ApplicationWindow, on_keep: FKeep)
where
    FKeep: Fn() + 'static,
{
    let alert = adw::AlertDialog::new(
        Some("File removed"),
        Some("This file was removed outside Omawrite. Keep your text as an unsaved document?"),
    );
    alert.add_response("keep", "Keep Mine");
    alert.set_response_appearance("keep", adw::ResponseAppearance::Suggested);
    alert.set_default_response(Some("keep"));
    alert.set_close_response("keep");

    alert.choose(window, gio::Cancellable::NONE, move |_| on_keep());
}

pub fn show_open_dialog<F>(window: &adw::ApplicationWindow, on_selected: F)
where
    F: Fn(PathBuf, String) + 'static,
{
    let dialog = gtk4::FileDialog::new();
    dialog.set_title("Open Markdown File");

    dialog.open(Some(window), gio::Cancellable::NONE, move |result| {
        if let Ok(file) = result {
            if let Some(path) = file.path() {
                if let Ok(content) = storage::load_file(&path) {
                    on_selected(path.to_path_buf(), content);
                }
            }
        }
    });
}

pub fn show_save_dialog<F>(window: &adw::ApplicationWindow, content: String, on_saved: F)
where
    F: Fn(PathBuf) + 'static,
{
    let dialog = gtk4::FileDialog::new();
    dialog.set_title("Save Markdown File");
    dialog.set_initial_name(Some("untitled.md"));

    dialog.save(Some(window), gio::Cancellable::NONE, move |result| {
        if let Ok(file) = result {
            if let Some(path) = file.path() {
                if storage::save_atomic(&path, &content).is_ok() {
                    on_saved(path.to_path_buf());
                }
            }
        }
    });
}
