use relm4::prelude::*;
use relm4::adw::prelude::*;
use gtk4::prelude::*;

struct AppModel;

#[derive(Debug)]
enum AppMsg {}

#[relm4::component]
impl SimpleComponent for AppModel {
    type Init = ();
    type Input = AppMsg;
    type Output = ();

    view! {
        adw::ApplicationWindow {
            set_title: Some("Omawrite (Rust)"),
            set_default_width: 800,
            set_default_height: 600,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                gtk::Label {
                    set_label: "✍️ Welcome to Omawrite-rs",
                    set_vexpand: true,
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = AppModel;
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }
}

fn main() {
    let app = RelmApp::new("io.omacom.omawrite-rs");
    app.run::<AppModel>(());
}
