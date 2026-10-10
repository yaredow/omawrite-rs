use gtk4::prelude::*;
use relm4::prelude::*;

pub struct FooterModel {
    pub word_count: usize,
    pub status_message: String,
}

#[derive(Debug)]
pub enum FooterInput {
    SetWordCount(usize),
    SetStatus(String),
    ClearStatus,
}

#[relm4::component(pub)]
impl SimpleComponent for FooterModel {
    type Init = ();
    type Input = FooterInput;
    type Output = ();

    view! {
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
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            word_count: 0,
            status_message: String::new(),
        };

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            FooterInput::SetWordCount(count) => {
                self.word_count = count;
            }
            FooterInput::SetStatus(status) => {
                self.status_message = status;
                let sender_clone = sender.clone();
                glib::timeout_add_local_once(std::time::Duration::from_secs(4), move || {
                    sender_clone.input(FooterInput::ClearStatus);
                });
            }
            FooterInput::ClearStatus => {
                self.status_message.clear();
            }
        }
    }
}
