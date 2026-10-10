mod app;
mod editor;
mod storage;
mod theme;
mod watcher;

use app::AppModel;
use relm4::RelmApp;

fn main() {
    let app = RelmApp::new("io.omacom.omawrite-rs");
    app.run::<AppModel>(());
}
