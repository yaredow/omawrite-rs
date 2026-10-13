use relm4::adw;
use relm4::adw::prelude::*;
use relm4::prelude::*;
use sourceview5::prelude::*;
use std::path::PathBuf;

use crate::core::config::{self, SidebarPosition};
use crate::core::metrics;
use crate::core::outline;
use crate::core::storage;
use crate::core::watcher;
use crate::ui::components::canvas;
use crate::ui::components::footer::{FooterInput, FooterModel};
use crate::ui::components::outline::{OutlineInput, OutlineModel, OutlineOutput};
use crate::ui::components::search::{SearchInput, SearchModel, SearchOutput};
use crate::ui::dialogs;
use crate::ui::theme;

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
    pub recovery_manager: Option<storage::RecoveryManager>,
    pub file_watcher: watcher::FileWatcher,
    pub external_dialog_open: bool,
    pub recovery_scheduled: bool,
    pub recovered_state: bool,
    pub close_after_save: bool,

    pub search_bar: Controller<SearchModel>,
    pub footer: Controller<FooterModel>,
    pub outline_sidebar: Controller<OutlineModel>,
    pub sidebar_visible: bool,
    pub sidebar_position: SidebarPosition,
    pub outline_scheduled: bool,
}

#[derive(Clone, Debug)]
pub enum AppMsg {
    DocumentChanged,
    SaveFile,
    SaveForClose,
    SaveAs,
    OpenFile,
    NewFile,
    FileLoaded(PathBuf, String),
    FileSaved(PathBuf),
    PromptClose(PendingAction),
    WindowCloseRequested,
    AutoSaveRecoveryDraft,
    ExternalFileChanged(watcher::FileChange),
    ReloadFromDisk,
    KeepLocalVersion,
    DiscardAndClose,
    ToggleFullscreen,

    ToggleOutline,
    UpdateOutline,
    CheckCursorPosition,
    OutlineAction(OutlineInput),
    OutlineResult(OutlineOutput),

    SearchAction(SearchInput),
    SearchResult(SearchOutput),

    FormatBold,
    FormatItalic,
    FormatLink,
    UpdateWordCount,
    ThemeChanged(bool),
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
            set_default_size: (820, 680),

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
                        connect_clicked => AppMsg::SearchAction(SearchInput::ToggleSearch),
                    },

                    pack_end = &gtk4::Button {
                        #[watch]
                        set_icon_name: "view-list-bullet-symbolic",
                        set_tooltip_text: Some("Document Outline (Ctrl+Shift+O)"),
                        connect_clicked => AppMsg::ToggleOutline,
                    },
                },

                #[local_ref]
                search_widget -> gtk4::Revealer {},

                adw::OverlaySplitView {
                    set_hexpand: true,
                    set_vexpand: true,
                    #[watch]
                    set_show_sidebar: model.sidebar_visible,
                    #[watch]
                    set_sidebar_position: match model.sidebar_position {
                        SidebarPosition::Left => gtk4::PackType::Start,
                        SidebarPosition::Right => gtk4::PackType::End,
                    },
                    set_min_sidebar_width: 200.0,
                    set_max_sidebar_width: 320.0,
                    set_sidebar_width_fraction: 0.22,

                    set_sidebar: Some(outline_widget),

                    #[wrap(Some)]
                    set_content = &gtk4::ScrolledWindow {
                        set_hexpand: true,
                        set_vexpand: true,

                        adw::Clamp {
                            set_maximum_size: 780,
                            set_tightening_threshold: 600,

                            #[local_ref]
                            editor_view -> sourceview5::View {},
                        },
                    },
                },

                #[local_ref]
                footer_widget -> gtk4::Box {},
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

        let editor_buffer = canvas::create_markdown_buffer(is_dark);
        let editor_view = sourceview5::View::with_buffer(&editor_buffer);
        canvas::configure_editor_view(&editor_view);

        let (search_settings, search_context) = canvas::setup_search(&editor_buffer);

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
            let shortcut = gtk4::Shortcut::new(Some(trigger), Some(action));
            controller.add_shortcut(shortcut);
        };

        add_shortcut(
            &shortcut_controller,
            "<Ctrl>s",
            AppMsg::SaveFile,
            sender.clone(),
        );
        add_shortcut(
            &shortcut_controller,
            "<Ctrl><Shift>s",
            AppMsg::SaveAs,
            sender.clone(),
        );
        add_shortcut(
            &shortcut_controller,
            "<Ctrl>o",
            AppMsg::OpenFile,
            sender.clone(),
        );
        add_shortcut(
            &shortcut_controller,
            "<Ctrl>n",
            AppMsg::NewFile,
            sender.clone(),
        );
        add_shortcut(
            &shortcut_controller,
            "<Ctrl>b",
            AppMsg::FormatBold,
            sender.clone(),
        );
        add_shortcut(
            &shortcut_controller,
            "<Ctrl>i",
            AppMsg::FormatItalic,
            sender.clone(),
        );
        add_shortcut(
            &shortcut_controller,
            "<Ctrl>k",
            AppMsg::FormatLink,
            sender.clone(),
        );
        add_shortcut(
            &shortcut_controller,
            "<Ctrl>f",
            AppMsg::SearchAction(SearchInput::ToggleSearch),
            sender.clone(),
        );
        add_shortcut(
            &shortcut_controller,
            "<Ctrl>h",
            AppMsg::SearchAction(SearchInput::ToggleReplace),
            sender.clone(),
        );
        add_shortcut(
            &shortcut_controller,
            "Escape",
            AppMsg::SearchAction(SearchInput::Close),
            sender.clone(),
        );
        add_shortcut(
            &shortcut_controller,
            "F11",
            AppMsg::ToggleFullscreen,
            sender.clone(),
        );
        add_shortcut(
            &shortcut_controller,
            "<Ctrl><Shift>o",
            AppMsg::ToggleOutline,
            sender.clone(),
        );
        add_shortcut(
            &shortcut_controller,
            "F9",
            AppMsg::ToggleOutline,
            sender.clone(),
        );
        add_shortcut(
            &shortcut_controller,
            "<Ctrl><Alt>o",
            AppMsg::OutlineAction(OutlineInput::TogglePosition),
            sender.clone(),
        );

        root.add_controller(shortcut_controller);

        let sender_for_cursor = sender.clone();
        editor_buffer.connect_cursor_position_notify(move |_| {
            sender_for_cursor.input(AppMsg::CheckCursorPosition);
        });

        let sender_for_theme = sender.clone();
        style_manager.connect_dark_notify(move |manager| {
            sender_for_theme.input(AppMsg::ThemeChanged(manager.is_dark()));
        });

        let search_bar = SearchModel::builder()
            .launch(())
            .forward(sender.input_sender(), AppMsg::SearchResult);
        let search_widget = search_bar.widget().clone();

        let footer = FooterModel::builder().launch(()).detach();
        let footer_widget = footer.widget().clone();

        let cfg = config::load_config();
        let sidebar_position = cfg.sidebar_position;
        let sidebar_visible = cfg.sidebar_visible;

        let outline_sidebar = OutlineModel::builder()
            .launch(sidebar_position)
            .forward(sender.input_sender(), AppMsg::OutlineResult);
        let outline_widget = outline_sidebar.widget().clone();

        let recovery_manager = storage::RecoveryManager::new();

        let mut initial_recovered_state = false;
        let mut initial_file_path = None;
        let mut initial_modified = false;

        if let Some(path) = init_path {
            if path.exists() {
                if let Ok(content) = storage::load_file(&path) {
                    editor_buffer.set_text(&content);
                    initial_file_path = Some(path);
                    initial_modified = false;
                }
            } else {
                initial_file_path = Some(path);
                initial_modified = false;
            }
        } else if let Some(mgr) = &recovery_manager {
            if let Some(snapshot) = &mgr.recovered_snapshot {
                editor_buffer.set_text(&snapshot.text);
                initial_file_path = snapshot.file_path.clone();
                initial_modified = true;
                initial_recovered_state = true;
            }
        }

        let mut file_watcher = watcher::FileWatcher::new();
        if let Some(path) = &initial_file_path {
            let sender_clone = sender.clone();
            let _ = file_watcher.watch(path, move |change| {
                sender_clone.input(AppMsg::ExternalFileChanged(change));
            });
        }

        let base_name = initial_file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Untitled".to_string());

        let initial_title = format!(
            "{}{}{}",
            if initial_recovered_state {
                "[Recovered] "
            } else if initial_modified {
                "* "
            } else {
                ""
            },
            base_name,
            " - Omawrite"
        );

        let initial_text = editor_buffer.text(
            &editor_buffer.start_iter(),
            &editor_buffer.end_iter(),
            false,
        );
        let initial_words = metrics::count_words(&initial_text);
        footer.emit(FooterInput::SetWordCount(initial_words));

        let initial_headings = outline::extract_headings(&initial_text);
        outline_sidebar.emit(OutlineInput::SetHeadings(initial_headings));

        let model = Self {
            editor_buffer,
            editor_view: editor_view.clone(),
            search_settings,
            search_context,
            is_dark,
            file_path: initial_file_path,
            modified: initial_modified,
            window_title: initial_title,
            doc_title: base_name,
            window: root.clone(),
            recovery_manager,
            file_watcher,
            external_dialog_open: false,
            recovery_scheduled: false,
            recovered_state: initial_recovered_state,
            close_after_save: false,
            search_bar,
            footer,
            outline_sidebar,
            sidebar_visible,
            sidebar_position,
            outline_scheduled: false,
        };

        let outline_widget = &outline_widget;
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            AppMsg::DocumentChanged => {
                if !self.modified {
                    self.modified = true;
                    self.refresh_titles();
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

                    let sender_for_metrics = sender.clone();
                    glib::timeout_add_local_once(
                        std::time::Duration::from_millis(120),
                        move || {
                            sender_for_metrics.input(AppMsg::UpdateWordCount);
                        },
                    );
                }

                if !self.outline_scheduled {
                    self.outline_scheduled = true;
                    let sender_for_outline = sender.clone();
                    glib::timeout_add_local_once(
                        std::time::Duration::from_millis(200),
                        move || {
                            sender_for_outline.input(AppMsg::UpdateOutline);
                        },
                    );
                }
            }
            AppMsg::ThemeChanged(is_dark) => {
                self.is_dark = is_dark;
                canvas::apply_style_scheme(&self.editor_buffer, is_dark);
                theme::apply_theme_mode(is_dark);
            }
            AppMsg::UpdateWordCount => {
                let text = self.get_editor_text();
                let count = metrics::count_words(&text);
                self.footer.emit(FooterInput::SetWordCount(count));
            }
            AppMsg::AutoSaveRecoveryDraft => {
                self.recovery_scheduled = false;
                if self.modified {
                    if let Some(mgr) = &self.recovery_manager {
                        let text = self.get_editor_text();
                        let _ = mgr.write_draft(self.file_path.as_deref(), &text);
                    }
                }
            }
            AppMsg::FormatBold => {
                canvas::format_bold(&self.editor_buffer);
            }
            AppMsg::FormatItalic => {
                canvas::format_italic(&self.editor_buffer);
            }
            AppMsg::FormatLink => {
                let clipboard = self.window.clipboard();
                let buffer = self.editor_buffer.clone();
                clipboard.read_text_async(gio::Cancellable::NONE, move |res| {
                    let url = res.ok().flatten();
                    let url_str = url.as_deref().map(str::trim);
                    canvas::format_link(&buffer, url_str);
                });
            }
            AppMsg::ToggleFullscreen => {
                if self.window.is_fullscreen() {
                    self.window.unfullscreen();
                } else {
                    self.window.fullscreen();
                }
            }
            AppMsg::NewFile => {
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
                    let sender_clone = sender.clone();
                    dialogs::show_open_dialog(&self.window, move |path, content| {
                        sender_clone.input(AppMsg::FileLoaded(path, content));
                    });
                }
            }
            AppMsg::FileLoaded(path, content) => {
                let headings = outline::extract_headings(&content);
                self.outline_sidebar
                    .emit(OutlineInput::SetHeadings(headings));
                self.editor_buffer.set_text(&content);
                self.file_path = Some(path);
                self.modified = false;
                self.recovered_state = false;
                let words = metrics::count_words(&content);
                self.footer.emit(FooterInput::SetWordCount(words));
                if let Some(mgr) = &self.recovery_manager {
                    mgr.clear();
                }
                self.watch_active_file(&sender);
                self.refresh_titles();
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
                let sender_clone = sender.clone();
                let content = self.get_editor_text();
                dialogs::show_save_dialog(&self.window, content, move |path| {
                    sender_clone.input(AppMsg::FileSaved(path));
                });
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
                self.footer
                    .emit(FooterInput::SetStatus(format!("Saved {}", base_name)));

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
                let pending_action = pending.clone();
                dialogs::prompt_discard(
                    &self.window,
                    move || {
                        if matches!(pending_action, PendingAction::Close) {
                            sender_clone.input(AppMsg::SaveForClose);
                        } else {
                            sender_clone.input(AppMsg::SaveFile);
                        }
                    },
                    move || match pending {
                        PendingAction::New => sender
                            .input(AppMsg::FileLoaded(PathBuf::from("Untitled"), String::new())),
                        PendingAction::Open => sender.input(AppMsg::OpenFile),
                        PendingAction::Close => sender.input(AppMsg::DiscardAndClose),
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
                            dialogs::prompt_external_modified(
                                &self.window,
                                is_dirty,
                                move || sender_clone.input(AppMsg::ReloadFromDisk),
                                move || sender.input(AppMsg::KeepLocalVersion),
                            );
                        }
                    }
                    watcher::FileChange::Deleted => {
                        self.external_dialog_open = true;
                        let sender_clone = sender.clone();
                        dialogs::prompt_external_deleted(&self.window, move || {
                            sender_clone.input(AppMsg::KeepLocalVersion);
                        });
                    }
                }
            }
            AppMsg::ReloadFromDisk => {
                self.external_dialog_open = false;
                if let Some(path) = self.file_path.clone() {
                    if let Ok(content) = storage::load_file(&path) {
                        let headings = outline::extract_headings(&content);
                        self.outline_sidebar
                            .emit(OutlineInput::SetHeadings(headings));
                        self.editor_buffer.set_text(&content);
                        self.modified = false;
                        self.recovered_state = false;
                        let words = metrics::count_words(&content);
                        self.footer.emit(FooterInput::SetWordCount(words));
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
            AppMsg::ToggleOutline => {
                self.sidebar_visible = !self.sidebar_visible;
                let mut config = config::load_config();
                config.sidebar_visible = self.sidebar_visible;
                config::save_config(&config);
            }
            AppMsg::UpdateOutline => {
                self.outline_scheduled = false;
                let text = self.get_editor_text();
                let headings = outline::extract_headings(&text);
                self.outline_sidebar
                    .emit(OutlineInput::SetHeadings(headings));
            }
            AppMsg::CheckCursorPosition => {
                let line = canvas::current_cursor_line(&self.editor_buffer);
                self.outline_sidebar.emit(OutlineInput::SetActiveLine(line));
            }
            AppMsg::OutlineAction(action) => {
                self.outline_sidebar.emit(action);
            }
            AppMsg::OutlineResult(result) => match result {
                OutlineOutput::JumpToLine(line) => {
                    canvas::scroll_to_line(&self.editor_view, &self.editor_buffer, line);
                }
                OutlineOutput::PositionChanged(pos) => {
                    self.sidebar_position = pos;
                    let mut config = config::load_config();
                    config.sidebar_position = pos;
                    config::save_config(&config);
                }
                OutlineOutput::CloseRequested => {
                    self.sidebar_visible = false;
                    let mut config = config::load_config();
                    config.sidebar_visible = false;
                    config::save_config(&config);
                }
            },
            AppMsg::SearchAction(action) => {
                self.search_bar.emit(action);
            }
            AppMsg::SearchResult(result) => match result {
                SearchOutput::SearchTextChanged(text) => {
                    if text.is_empty() {
                        self.search_settings.set_search_text(None);
                    } else {
                        self.search_settings.set_search_text(Some(&text));
                    }
                }
                SearchOutput::FindNext => {
                    canvas::find_next(&self.editor_view, &self.editor_buffer, &self.search_context);
                }
                SearchOutput::FindPrev => {
                    canvas::find_prev(&self.editor_view, &self.editor_buffer, &self.search_context);
                }
                SearchOutput::ReplaceCurrent(replacement) => {
                    canvas::replace_current(
                        &self.editor_view,
                        &self.editor_buffer,
                        &self.search_context,
                        &replacement,
                    );
                }
                SearchOutput::ReplaceAll(replacement) => {
                    canvas::replace_all(&self.search_context, &replacement);
                }
                SearchOutput::Closed => {
                    self.search_settings.set_search_text(None);
                    self.editor_view.grab_focus();
                }
            },
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
        self.footer.emit(FooterInput::SetWordCount(0));
        self.footer.emit(FooterInput::SetStatus(String::new()));
        self.outline_sidebar
            .emit(OutlineInput::SetHeadings(Vec::new()));
        if let Some(mgr) = &self.recovery_manager {
            mgr.clear();
        }
        self.file_watcher.unwatch();
        self.refresh_titles();
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
}
