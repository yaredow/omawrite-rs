use gtk4::prelude::*;
use relm4::prelude::*;

pub struct SearchModel {
    pub visible: bool,
    pub replace_mode: bool,
    pub search_text: String,
    pub replace_text: String,
}

#[derive(Clone, Debug)]
pub enum SearchInput {
    ToggleSearch,
    ToggleReplace,
    Close,
    SearchChanged(String),
    ReplaceChanged(String),
    TriggerNext,
    TriggerPrev,
    TriggerReplaceCurrent,
    TriggerReplaceAll,
}

#[derive(Clone, Debug)]
pub enum SearchOutput {
    SearchTextChanged(String),
    FindNext,
    FindPrev,
    ReplaceCurrent(String),
    ReplaceAll(String),
    Closed,
}

#[relm4::component(pub)]
impl SimpleComponent for SearchModel {
    type Init = ();
    type Input = SearchInput;
    type Output = SearchOutput;

    view! {
        gtk4::Revealer {
            #[watch]
            set_reveal_child: model.visible,
            set_transition_type: gtk4::RevealerTransitionType::SlideDown,

            gtk4::Box {
                set_orientation: gtk4::Orientation::Vertical,
                set_spacing: 6,
                add_css_class: "search-bar",

                // Row 1: Search controls
                gtk4::Box {
                    set_orientation: gtk4::Orientation::Horizontal,
                    set_spacing: 6,

                    gtk4::Button {
                        #[watch]
                        set_icon_name: if model.replace_mode {
                            "pan-down-symbolic"
                        } else {
                            "pan-end-symbolic"
                        },
                        set_tooltip_text: Some("Toggle Replace (Ctrl+H)"),
                        connect_clicked => SearchInput::ToggleReplace,
                    },

                    #[name = "search_entry"]
                    gtk4::SearchEntry {
                        set_hexpand: true,
                        set_placeholder_text: Some("Search (Ctrl+F)"),
                        connect_search_changed[sender] => move |entry| {
                            sender.input(SearchInput::SearchChanged(entry.text().to_string()));
                        },
                        connect_activate => SearchInput::TriggerNext,
                    },

                    gtk4::Button {
                        set_icon_name: "go-up-symbolic",
                        set_tooltip_text: Some("Previous Match (Shift+Enter)"),
                        connect_clicked => SearchInput::TriggerPrev,
                    },

                    gtk4::Button {
                        set_icon_name: "go-down-symbolic",
                        set_tooltip_text: Some("Next Match (Enter)"),
                        connect_clicked => SearchInput::TriggerNext,
                    },

                    gtk4::Button {
                        set_icon_name: "window-close-symbolic",
                        set_tooltip_text: Some("Close (Esc)"),
                        connect_clicked => SearchInput::Close,
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

                        gtk4::Box {
                            set_size_request: (34, -1),
                        },

                        gtk4::Entry {
                            set_hexpand: true,
                            set_placeholder_text: Some("Replace..."),
                            connect_changed[sender] => move |entry| {
                                sender.input(SearchInput::ReplaceChanged(entry.text().to_string()));
                            },
                            connect_activate => SearchInput::TriggerReplaceCurrent,
                        },

                        gtk4::Button {
                            set_label: "Replace",
                            set_tooltip_text: Some("Replace Current Match"),
                            connect_clicked => SearchInput::TriggerReplaceCurrent,
                        },

                        gtk4::Button {
                            set_label: "Replace All",
                            set_tooltip_text: Some("Replace All Matches"),
                            connect_clicked => SearchInput::TriggerReplaceAll,
                        },
                    },
                },
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            visible: false,
            replace_mode: false,
            search_text: String::new(),
            replace_text: String::new(),
        };

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            SearchInput::ToggleSearch => {
                self.visible = !self.visible;
                if !self.visible {
                    self.search_text.clear();
                    let _ = sender.output(SearchOutput::SearchTextChanged(String::new()));
                    let _ = sender.output(SearchOutput::Closed);
                }
            }
            SearchInput::ToggleReplace => {
                if !self.visible {
                    self.visible = true;
                    self.replace_mode = true;
                } else {
                    self.replace_mode = !self.replace_mode;
                }
            }
            SearchInput::Close => {
                self.visible = false;
                self.search_text.clear();
                let _ = sender.output(SearchOutput::SearchTextChanged(String::new()));
                let _ = sender.output(SearchOutput::Closed);
            }
            SearchInput::SearchChanged(text) => {
                self.search_text = text.clone();
                let _ = sender.output(SearchOutput::SearchTextChanged(text));
            }
            SearchInput::ReplaceChanged(text) => {
                self.replace_text = text;
            }
            SearchInput::TriggerNext => {
                let _ = sender.output(SearchOutput::FindNext);
            }
            SearchInput::TriggerPrev => {
                let _ = sender.output(SearchOutput::FindPrev);
            }
            SearchInput::TriggerReplaceCurrent => {
                let _ = sender.output(SearchOutput::ReplaceCurrent(self.replace_text.clone()));
            }
            SearchInput::TriggerReplaceAll => {
                let _ = sender.output(SearchOutput::ReplaceAll(self.replace_text.clone()));
            }
        }
    }
}
