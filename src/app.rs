use gtk4::prelude::*;
use relm4::adw;
use relm4::prelude::*;

use crate::editor;

pub struct AppModel {
    pub editor_buffer: sourceview5::Buffer,
    pub is_dark: bool,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum AppMsg {
    FormatBold,
    FormatItalic,
    FormatLink,
    ThemeChanged(bool),
}

#[relm4::component(pub)]
impl SimpleComponent for AppModel {
    type Init = ();
    type Input = AppMsg;
    type Output = ();

    view! {
        adw::ApplicationWindow {
            set_title: Some("Untitled - Omawrite"),
            set_default_width: 960,
            set_default_height: 700,

            gtk4::Box {
                set_orientation: gtk4::Orientation::Vertical,

                adw::HeaderBar {
                    #[wrap(Some)]
                    set_title_widget = &adw::WindowTitle {
                        set_title: "Untitled",
                        set_subtitle: "Omawrite",
                    },
                },

                gtk4::ScrolledWindow {
                    set_hscrollbar_policy: gtk4::PolicyType::Never,
                    set_vscrollbar_policy: gtk4::PolicyType::Automatic,
                    set_vexpand: true,
                    set_hexpand: true,

                    adw::Clamp {
                        set_maximum_size: 780,
                        set_tightening_threshold: 620,

                        #[local_ref]
                        editor_view -> sourceview5::View {},
                    },
                },
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let style_manager = adw::StyleManager::default();
        let is_dark = style_manager.is_dark();

        let editor_buffer = editor::create_markdown_buffer(is_dark);
        let editor_view = sourceview5::View::with_buffer(&editor_buffer);
        editor::configure_editor_view(&editor_view);

        let shortcut_controller = gtk4::ShortcutController::new();
        shortcut_controller.set_scope(gtk4::ShortcutScope::Local);

        let add_shortcut = |controller: &gtk4::ShortcutController, trigger_str: &str, action_fn: Box<dyn Fn() -> glib::Propagation + 'static>| {
            let trigger = gtk4::ShortcutTrigger::parse_string(trigger_str).expect("valid accelerator");
            let action = gtk4::CallbackAction::new(move |_widget, _args| {
                action_fn()
            });
            controller.add_shortcut(gtk4::Shortcut::new(Some(trigger), Some(action)));
        };

        let buffer_bold = editor_buffer.clone();
        add_shortcut(&shortcut_controller, "<Control>b", Box::new(move || {
            editor::format_bold(&buffer_bold);
            glib::Propagation::Stop
        }));

        let buffer_italic = editor_buffer.clone();
        add_shortcut(&shortcut_controller, "<Control>i", Box::new(move || {
            editor::format_italic(&buffer_italic);
            glib::Propagation::Stop
        }));

        let buffer_link = editor_buffer.clone();
        add_shortcut(&shortcut_controller, "<Control>k", Box::new(move || {
            editor::format_link(&buffer_link, None);
            glib::Propagation::Stop
        }));

        editor_view.add_controller(shortcut_controller);

        let sender_for_theme = sender.clone();
        style_manager.connect_dark_notify(move |manager| {
            sender_for_theme.input(AppMsg::ThemeChanged(manager.is_dark()));
        });

        let model = AppModel {
            editor_buffer,
            is_dark,
        };

        let widgets = view_output!();
        editor_view.grab_focus();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
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
        }
    }
}
