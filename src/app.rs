use gtk4::prelude::*;
use relm4::adw;
use relm4::adw::prelude::*;
use relm4::prelude::*;
use sourceview5::prelude::*;
use std::path::PathBuf;

use crate::editor;
use crate::storage;
use crate::watcher;

#[derive(Clone, Debug)]
pub enum PendingAction {
    New,
    Open,
    Close,
}

pub struct AppModel {
    pub editor_buffer: sourceview5::Buffer,
    pub editor_view: sourceview5::View,
    pub search_settings: sourceview5::SearchSettings,
    pub search_context: sourceview5::SearchContext,
    pub is_dark: bool,
    pub file_path: Option<PathBuf>,
    pub modified: bool,
    pub window_title: String,
    pub doc_title: String,
    pub window: adw::ApplicationWindow,
    pub search_visible: bool,
    pub replace_mode: bool,
    pub search_text: String,
    pub replace_text: String,
    pub recovery_manager: Option<storage::RecoveryManager>,
    pub file_watcher: watcher::FileWatcher,
    pub external_dialog_open: bool,
    pub recovery_scheduled: bool,
    pub recovered_state: bool,
    pub close_after_save: bool,
    pub word_count: usize,
    pub word_count_scheduled: bool,
    pub status_message: String,
    pub is_fullscreen: bool,
}

#[derive(Clone, Debug)]
pub enum AppMsg {
    FormatBold,
    FormatItalic,
    FormatLink,
    ThemeChanged(bool),
    DocumentChanged,
    NewDocument,
    OpenFile,
    FileLoaded(PathBuf, String),
    SaveFile,
    SaveForClose,
    SaveAs,
    FileSaved(PathBuf),
    PromptClose(PendingAction),
    WindowCloseRequested,
    AutoSaveRecoveryDraft,
    ExternalFileChanged(watcher::FileChange),
    ReloadFromDisk,
    KeepLocalVersion,
    DiscardAndClose,
    ToggleSearch,
    ToggleReplace,
    CloseSearch,
    SearchTextChanged(String),
    ReplaceTextChanged(String),
    FindNext,
    FindPrev,
    DoReplaceCurrent,
    DoReplaceAll,
    UpdateWordCount,
    ClearStatusMessage,
    ToggleFullscreen,
}

#[relm4::component(pub)]
impl SimpleComponent for AppModel {
    type Init = Option<PathBuf>;
    type Input = AppMsg;
    type Output = ();

    view! {
        adw::ApplicationWindow {
            #[watch]
            set_title: Some(&model.window_title),
            set_default_width: 960,
            set_default_height: 700,
            connect_close_request[sender] => move |_| {
                sender.input(AppMsg::WindowCloseRequested);
                glib::Propagation::Stop
            },

            gtk4::Box {
                set_orientation: gtk4::Orientation::Vertical,

                adw::HeaderBar {
                    #[wrap(Some)]
                    set_title_widget = &adw::WindowTitle {
                        #[watch]
                        set_title: &model.doc_title,
                        #[watch]
                        set_subtitle: if model.recovered_state {
                            "Recovered"
                        } else if model.modified {
                            "Modified"
                        } else {
                            "Saved"
                        },
                    },

                    pack_end = &gtk4::Button {
                        set_icon_name: "edit-find-symbolic",
                        set_tooltip_text: Some("Search & Replace (Ctrl+F)"),
                        connect_clicked => AppMsg::ToggleSearch,
                    },
                },

                // Search & Replace sliding overlay
                gtk4::Revealer {
                    #[watch]
                    set_reveal_child: model.search_visible,
                    set_transition_type: gtk4::RevealerTransitionType::SlideDown,

                    gtk4::Box {
                        set_orientation: gtk4::Orientation::Vertical,
                        set_spacing: 6,
                        set_margin_start: 18,
                        set_margin_end: 18,
                        set_margin_top: 8,
                        set_margin_bottom: 8,
                        add_css_class: "toolbar",

                        // Row 1: Search Query + Navigation
                        gtk4::Box {
                            set_orientation: gtk4::Orientation::Horizontal,
                            set_spacing: 6,

                            #[name = "search_entry"]
                            gtk4::SearchEntry {
                                set_placeholder_text: Some("Find..."),
                                set_hexpand: true,
                                connect_search_changed[sender] => move |entry| {
                                    sender.input(AppMsg::SearchTextChanged(entry.text().to_string()));
                                },
                                connect_activate => AppMsg::FindNext,
                            },

                            gtk4::Button {
                                set_icon_name: "go-up-symbolic",
                                set_tooltip_text: Some("Previous Match (Shift+Enter)"),
                                connect_clicked => AppMsg::FindPrev,
                            },

                            gtk4::Button {
                                set_icon_name: "go-down-symbolic",
                                set_tooltip_text: Some("Next Match (Enter)"),
                                connect_clicked => AppMsg::FindNext,
                            },

                            gtk4::Button {
                                set_icon_name: "window-close-symbolic",
                                set_tooltip_text: Some("Close (Esc)"),
                                connect_clicked => AppMsg::CloseSearch,
                            },
                        },

                        // Row 2: Replace controls
                        gtk4::Revealer {
                            #[watch]
                            set_reveal_child: model.replace_mode,
                            set_transition_type: gtk4::RevealerTransitionType::SlideDown,

                            gtk4::Box {
                                set_orientation: gtk4::Orientation::Horizontal,
                                set_spacing: 6,
                                set_margin_top: 4,

                                gtk4::Entry {
                                    set_placeholder_text: Some("Replace with..."),
                                    set_hexpand: true,
                                    connect_changed[sender] => move |entry| {
                                        sender.input(AppMsg::ReplaceTextChanged(entry.text().to_string()));
                                    },
                                    connect_activate => AppMsg::DoReplaceCurrent,
                                },

                                gtk4::Button {
                                    set_label: "Replace",
                                    connect_clicked => AppMsg::DoReplaceCurrent,
                                },

                                gtk4::Button {
                                    set_label: "Replace All",
                                    connect_clicked => AppMsg::DoReplaceAll,
                                },
                            },
                        },
                    },
                },

                // Centered distraction-free writing canvas
                gtk4::ScrolledWindow {
                    set_vexpand: true,
                    set_hexpand: true,

                    adw::Clamp {
                        set_maximum_size: 780,
                        set_tightening_threshold: 600,

                        #[local_ref]
                        editor_view -> sourceview5::View {},
                    },
                },

                // Footer Status Bar with Word Count
                gtk4::Box {
                    set_orientation: gtk4::Orientation::Horizontal,
                    set_spacing: 12,
                    add_css_class: "footer-bar",

                    gtk4::Label {
                        #[watch]
                        set_text: &model.status_message,
                        #[watch]
                        set_visible: !model.status_message.is_empty(),
                        add_css_class: "status-label",
                    },

                    gtk4::Box {
                        set_hexpand: true,
                    },

                    gtk4::Label {
                        #[watch]
                        set_text: &format!(
                            "{} {}",
                            model.word_count,
                            if model.word_count == 1 { "Word" } else { "Words" }
                        ),
                        add_css_class: "word-count",
                    },
                },
            },
        }
    }

    fn init(
        init_path: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let style_manager = adw::StyleManager::default();
        let is_dark = style_manager.is_dark();

        let editor_buffer = editor::create_markdown_buffer(is_dark);
        let editor_view = sourceview5::View::with_buffer(&editor_buffer);
        editor::configure_editor_view(&editor_view);

        let (search_settings, search_context) = editor::setup_search(&editor_buffer);

        let sender_for_changes = sender.clone();
        editor_buffer.connect_changed(move |_| {
            sender_for_changes.input(AppMsg::DocumentChanged);
        });

        let shortcut_controller = gtk4::ShortcutController::new();
        shortcut_controller.set_scope(gtk4::ShortcutScope::Local);

        let add_shortcut = |controller: &gtk4::ShortcutController,
                            trigger_str: &str,
                            msg: AppMsg,
                            sender: ComponentSender<Self>| {
            let trigger =
                gtk4::ShortcutTrigger::parse_string(trigger_str).expect("valid accelerator");
            let action = gtk4::CallbackAction::new(move |_widget, _args| {
                sender.input(msg.clone());
                glib::Propagation::Stop
            });
            controller.add_shortcut(gtk4::Shortcut::new(Some(trigger), Some(action)));
        };

        add_shortcut(
            &shortcut_controller,
            "<Control>b",
            AppMsg::FormatBold,
            sender.clone(),
        );
        add_shortcut(
            &shortcut_controller,
            "<Control>i",
            AppMsg::FormatItalic,
            sender.clone(),
        );
        add_shortcut(
            &shortcut_controller,
            "<Control>k",
            AppMsg::FormatLink,
            sender.clone(),
        );
        add_shortcut(
            &shortcut_controller,
            "<Control>s",
            AppMsg::SaveFile,
            sender.clone(),
        );
        add_shortcut(
            &shortcut_controller,
            "<Control><Shift>s",
            AppMsg::SaveAs,
            sender.clone(),
        );
        add_shortcut(
            &shortcut_controller,
            "<Control>o",
            AppMsg::OpenFile,
            sender.clone(),
        );
        add_shortcut(
            &shortcut_controller,
            "<Control>n",
            AppMsg::NewDocument,
            sender.clone(),
        );
        add_shortcut(
            &shortcut_controller,
            "<Control>f",
            AppMsg::ToggleSearch,
            sender.clone(),
        );
        add_shortcut(
            &shortcut_controller,
            "<Control>h",
            AppMsg::ToggleReplace,
            sender.clone(),
        );
        add_shortcut(
            &shortcut_controller,
            "Escape",
            AppMsg::CloseSearch,
            sender.clone(),
        );
        add_shortcut(
            &shortcut_controller,
            "F11",
            AppMsg::ToggleFullscreen,
            sender.clone(),
        );

        root.add_controller(shortcut_controller);

        let sender_for_theme = sender.clone();
        style_manager.connect_dark_notify(move |manager| {
            sender_for_theme.input(AppMsg::ThemeChanged(manager.is_dark()));
        });

        let recovery_manager = storage::RecoveryManager::new();
        let mut file_path = None;
        let mut modified = false;
        let mut recovered_state = false;
        let mut status_message = String::new();

        if let Some(path) = init_path {
            if path.exists() {
                if let Ok(content) = storage::load_file(&path) {
                    editor_buffer.set_text(&content);
                    let base_name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "document.md".to_string());
                    status_message = format!("Loaded {}", base_name);
                    file_path = Some(path);
                    modified = false;
                }
            } else {
                file_path = Some(path);
                modified = true;
            }
        } else if let Some(mgr) = &recovery_manager {
            if let Some(snapshot) = &mgr.recovered_snapshot {
                editor_buffer.set_text(&snapshot.text);
                file_path = snapshot.file_path.clone();
                modified = true;
                recovered_state = true;
                status_message = "Recovered unsaved changes".to_string();
            }
        }

        let mut file_watcher = watcher::FileWatcher::new();
        if let Some(path) = &file_path {
            let sender_for_watch = sender.clone();
            let _ = file_watcher.watch(path, move |change| {
                sender_for_watch.input(AppMsg::ExternalFileChanged(change));
            });
        }

        let base_name = file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Untitled".to_string());

        let window_title = format!(
            "{}{} - Omawrite",
            if modified { "* " } else { "" },
            base_name
        );
        let doc_title = base_name;
        let word_count = editor::count_words(&editor_buffer.text(
            &editor_buffer.start_iter(),
            &editor_buffer.end_iter(),
            false,
        ));

        let model = AppModel {
            editor_buffer,
            editor_view: editor_view.clone(),
            search_settings,
            search_context,
            is_dark,
            file_path,
            modified,
            window_title,
            doc_title,
            window: root.clone(),
            search_visible: false,
            replace_mode: false,
            search_text: String::new(),
            replace_text: String::new(),
            recovery_manager,
            file_watcher,
            external_dialog_open: false,
            recovery_scheduled: false,
            recovered_state,
            close_after_save: false,
            word_count,
            word_count_scheduled: false,
            status_message,
            is_fullscreen: false,
        };

        let editor_view = &editor_view;
        let widgets = view_output!();
        editor_view.grab_focus();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            AppMsg::FormatBold => {
                editor::format_bold(&self.editor_buffer);
            }
            AppMsg::FormatItalic => {
                editor::format_italic(&self.editor_buffer);
            }
            AppMsg::FormatLink => {
                editor::format_link(&self.editor_buffer, None);
            }
            AppMsg::ThemeChanged(is_dark) => {
                self.is_dark = is_dark;
                editor::apply_style_scheme(&self.editor_buffer, is_dark);
            }
            AppMsg::DocumentChanged => {
                self.recovered_state = false;
                if !self.modified {
                    self.modified = true;
                    self.refresh_titles();
                }

                if !self.word_count_scheduled {
                    self.word_count_scheduled = true;
                    let sender_clone = sender.clone();
                    glib::timeout_add_local_once(
                        std::time::Duration::from_millis(120),
                        move || {
                            sender_clone.input(AppMsg::UpdateWordCount);
                        },
                    );
                }

                if !self.recovery_scheduled {
                    self.recovery_scheduled = true;
                    let sender_clone = sender.clone();
                    glib::timeout_add_local_once(
                        std::time::Duration::from_millis(750),
                        move || {
                            sender_clone.input(AppMsg::AutoSaveRecoveryDraft);
                        },
                    );
                }
            }
            AppMsg::UpdateWordCount => {
                self.word_count_scheduled = false;
                self.word_count = editor::count_words(&self.get_editor_text());
            }
            AppMsg::ClearStatusMessage => {
                self.status_message.clear();
            }
            AppMsg::ToggleFullscreen => {
                self.is_fullscreen = !self.is_fullscreen;
                if self.is_fullscreen {
                    self.window.fullscreen();
                } else {
                    self.window.unfullscreen();
                }
            }
            AppMsg::AutoSaveRecoveryDraft => {
                self.recovery_scheduled = false;
                if self.modified {
                    if let Some(mgr) = &self.recovery_manager {
                        let _ = mgr.write_draft(self.file_path.as_deref(), &self.get_editor_text());
                    }
                }
            }
            AppMsg::NewDocument => {
                if self.modified {
                    sender.input(AppMsg::PromptClose(PendingAction::New));
                } else {
                    self.clear_document();
                }
            }
            AppMsg::OpenFile => {
                if self.modified {
                    sender.input(AppMsg::PromptClose(PendingAction::Open));
                } else {
                    self.show_open_dialog(sender);
                }
            }
            AppMsg::FileLoaded(path, content) => {
                let base_name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "document.md".to_string());
                self.editor_buffer.set_text(&content);
                self.file_path = Some(path);
                self.modified = false;
                self.recovered_state = false;
                self.word_count = editor::count_words(&content);
                if let Some(mgr) = &self.recovery_manager {
                    mgr.clear();
                }
                self.watch_active_file(&sender);
                self.refresh_titles();
                self.set_transient_status(format!("Loaded {}", base_name), &sender);
            }
            AppMsg::SaveFile => {
                if let Some(path) = self.file_path.clone() {
                    let content = self.get_editor_text();
                    if storage::save_atomic(&path, &content).is_ok() {
                        sender.input(AppMsg::FileSaved(path));
                    }
                } else {
                    sender.input(AppMsg::SaveAs);
                }
            }
            AppMsg::SaveForClose => {
                self.close_after_save = true;
                sender.input(AppMsg::SaveFile);
            }
            AppMsg::SaveAs => {
                self.show_save_dialog(sender);
            }
            AppMsg::FileSaved(path) => {
                let base_name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "document.md".to_string());
                self.file_path = Some(path);
                self.modified = false;
                self.recovered_state = false;
                if let Some(mgr) = &self.recovery_manager {
                    mgr.clear();
                }
                self.watch_active_file(&sender);
                self.refresh_titles();
                self.set_transient_status(format!("Saved {}", base_name), &sender);

                if self.close_after_save {
                    self.window.destroy();
                }
            }
            AppMsg::WindowCloseRequested => {
                if self.modified {
                    sender.input(AppMsg::PromptClose(PendingAction::Close));
                } else {
                    if let Some(mgr) = &self.recovery_manager {
                        mgr.clear();
                    }
                    self.window.destroy();
                }
            }
            AppMsg::PromptClose(pending) => {
                let sender_clone = sender.clone();
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

                let pending_action = pending.clone();
                alert.choose(
                    &self.window,
                    gio::Cancellable::NONE,
                    move |response| match response.as_str() {
                        "save" => {
                            if matches!(pending_action, PendingAction::Close) {
                                sender_clone.input(AppMsg::SaveForClose);
                            } else {
                                sender_clone.input(AppMsg::SaveFile);
                            }
                        }
                        "discard" => match pending_action {
                            PendingAction::New => sender_clone.input(AppMsg::FileLoaded(
                                PathBuf::from("Untitled"),
                                String::new(),
                            )),
                            PendingAction::Open => sender_clone.input(AppMsg::OpenFile),
                            PendingAction::Close => sender_clone.input(AppMsg::DiscardAndClose),
                        },
                        _ => {}
                    },
                );
            }
            AppMsg::DiscardAndClose => {
                if let Some(mgr) = &self.recovery_manager {
                    mgr.clear();
                }
                self.window.destroy();
            }
            AppMsg::ExternalFileChanged(change) => {
                if self.external_dialog_open {
                    return;
                }

                match change {
                    watcher::FileChange::Modified => {
                        if let Some(path) = &self.file_path {
                            if let Ok(disk_content) = storage::load_file(path) {
                                if disk_content == self.get_editor_text() {
                                    return;
                                }
                            }

                            self.external_dialog_open = true;
                            let sender_clone = sender.clone();
                            let is_dirty = self.modified;
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
                            alert.set_response_appearance(
                                "reload",
                                adw::ResponseAppearance::Suggested,
                            );
                            alert.set_default_response(Some("reload"));
                            alert.set_close_response("keep");

                            alert.choose(&self.window, gio::Cancellable::NONE, move |response| {
                                match response.as_str() {
                                    "reload" => sender_clone.input(AppMsg::ReloadFromDisk),
                                    _ => sender_clone.input(AppMsg::KeepLocalVersion),
                                }
                            });
                        }
                    }
                    watcher::FileChange::Deleted => {
                        self.external_dialog_open = true;
                        let sender_clone = sender.clone();
                        let alert = adw::AlertDialog::new(
                            Some("File removed"),
                            Some("This file was removed outside Omawrite. Keep your text as an unsaved document?"),
                        );
                        alert.add_response("keep", "Keep Mine");
                        alert.set_response_appearance("keep", adw::ResponseAppearance::Suggested);
                        alert.set_default_response(Some("keep"));
                        alert.set_close_response("keep");

                        alert.choose(&self.window, gio::Cancellable::NONE, move |_| {
                            sender_clone.input(AppMsg::KeepLocalVersion);
                        });
                    }
                }
            }
            AppMsg::ReloadFromDisk => {
                self.external_dialog_open = false;
                if let Some(path) = self.file_path.clone() {
                    if let Ok(content) = storage::load_file(&path) {
                        self.editor_buffer.set_text(&content);
                        self.modified = false;
                        self.recovered_state = false;
                        self.word_count = editor::count_words(&content);
                        if let Some(mgr) = &self.recovery_manager {
                            mgr.clear();
                        }
                        self.refresh_titles();
                    }
                }
            }
            AppMsg::KeepLocalVersion => {
                self.external_dialog_open = false;
                if let Some(path) = &self.file_path {
                    if !path.exists() {
                        self.file_path = None;
                        self.file_watcher.unwatch();
                    }
                }
                self.modified = true;
                self.refresh_titles();
            }
            AppMsg::ToggleSearch => {
                self.search_visible = !self.search_visible;
                if !self.search_visible {
                    self.search_settings.set_search_text(None);
                    self.editor_view.grab_focus();
                }
            }
            AppMsg::ToggleReplace => {
                if !self.search_visible {
                    self.search_visible = true;
                    self.replace_mode = true;
                } else {
                    self.replace_mode = !self.replace_mode;
                }
            }
            AppMsg::CloseSearch => {
                self.search_visible = false;
                self.search_settings.set_search_text(None);
                self.editor_view.grab_focus();
            }
            AppMsg::SearchTextChanged(text) => {
                self.search_text = text.clone();
                if text.is_empty() {
                    self.search_settings.set_search_text(None);
                } else {
                    self.search_settings.set_search_text(Some(&text));
                }
            }
            AppMsg::ReplaceTextChanged(text) => {
                self.replace_text = text;
            }
            AppMsg::FindNext => {
                editor::find_next(&self.editor_view, &self.editor_buffer, &self.search_context);
            }
            AppMsg::FindPrev => {
                editor::find_prev(&self.editor_view, &self.editor_buffer, &self.search_context);
            }
            AppMsg::DoReplaceCurrent => {
                editor::replace_current(
                    &self.editor_view,
                    &self.editor_buffer,
                    &self.search_context,
                    &self.replace_text,
                );
            }
            AppMsg::DoReplaceAll => {
                editor::replace_all(&self.search_context, &self.replace_text);
            }
        }
    }
}

impl AppModel {
    fn refresh_titles(&mut self) {
        let base_name = self
            .file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Untitled".to_string());

        self.doc_title = base_name.clone();
        self.window_title = format!(
            "{}{} - Omawrite",
            if self.modified { "* " } else { "" },
            base_name
        );
    }

    fn clear_document(&mut self) {
        self.editor_buffer.set_text("");
        self.file_path = None;
        self.modified = false;
        self.recovered_state = false;
        self.word_count = 0;
        self.status_message.clear();
        if let Some(mgr) = &self.recovery_manager {
            mgr.clear();
        }
        self.file_watcher.unwatch();
        self.refresh_titles();
    }

    fn set_transient_status(&mut self, msg: String, sender: &ComponentSender<Self>) {
        self.status_message = msg;
        let sender_clone = sender.clone();
        glib::timeout_add_local_once(std::time::Duration::from_secs(4), move || {
            sender_clone.input(AppMsg::ClearStatusMessage);
        });
    }

    fn watch_active_file(&mut self, sender: &ComponentSender<Self>) {
        if let Some(path) = &self.file_path {
            let sender_clone = sender.clone();
            let _ = self.file_watcher.watch(path, move |change| {
                sender_clone.input(AppMsg::ExternalFileChanged(change));
            });
        } else {
            self.file_watcher.unwatch();
        }
    }

    fn get_editor_text(&self) -> String {
        self.editor_buffer
            .text(
                &self.editor_buffer.start_iter(),
                &self.editor_buffer.end_iter(),
                false,
            )
            .to_string()
    }

    fn show_open_dialog(&self, sender: ComponentSender<Self>) {
        let dialog = gtk4::FileDialog::new();
        dialog.set_title("Open Markdown File");

        dialog.open(Some(&self.window), gio::Cancellable::NONE, move |result| {
            if let Ok(file) = result {
                if let Some(path) = file.path() {
                    if let Ok(content) = storage::load_file(&path) {
                        sender.input(AppMsg::FileLoaded(path, content));
                    }
                }
            }
        });
    }

    fn show_save_dialog(&self, sender: ComponentSender<Self>) {
        let dialog = gtk4::FileDialog::new();
        dialog.set_title("Save Markdown File");
        dialog.set_initial_name(Some("untitled.md"));

        let content = self.get_editor_text();
        dialog.save(Some(&self.window), gio::Cancellable::NONE, move |result| {
            if let Ok(file) = result {
                if let Some(path) = file.path() {
                    if storage::save_atomic(&path, &content).is_ok() {
                        sender.input(AppMsg::FileSaved(path));
                    }
                }
            }
        });
    }
}
