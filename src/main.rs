mod app;
mod editor;
mod storage;
mod theme;
mod watcher;

use app::AppModel;
use relm4::RelmApp;

fn ensure_environment() {
    if std::env::var("_OMAWRITE_REEXEC").is_ok() {
        return;
    }

    let schema_dir = option_env!("NIX_GSETTINGS_SCHEMA_DIR");
    let schema_path = option_env!("NIX_GSETTINGS_PATH");
    let gio_modules = option_env!("NIX_GIO_MODULES");

    let current_xdg = std::env::var("XDG_DATA_DIRS").unwrap_or_default();
    let current_schema_dir = std::env::var("GSETTINGS_SCHEMA_DIR").unwrap_or_default();
    let current_gio = std::env::var("GIO_EXTRA_MODULES").unwrap_or_default();

    let needs_schema_dir = if let Some(dir) = schema_dir {
        !current_schema_dir.contains(dir)
    } else {
        false
    };

    let needs_xdg = if let Some(path) = schema_path {
        !current_xdg.contains(path)
    } else {
        false
    };

    let needs_gio = if let Some(gio) = gio_modules {
        !current_gio.contains(gio)
    } else {
        false
    };

    if !needs_schema_dir && !needs_xdg && !needs_gio {
        return;
    }

    if let Ok(exe) = std::env::current_exe() {
        use std::os::unix::process::CommandExt;
        let mut cmd = std::process::Command::new(exe);
        cmd.args(std::env::args().skip(1));
        cmd.env("_OMAWRITE_REEXEC", "1");

        if let Some(dir) = schema_dir {
            let new_dir = if current_schema_dir.is_empty() {
                dir.to_string()
            } else {
                format!("{}:{}", dir, current_schema_dir)
            };
            cmd.env("GSETTINGS_SCHEMA_DIR", new_dir);
        }

        if let Some(path) = schema_path {
            let new_xdg = if current_xdg.is_empty() {
                path.to_string()
            } else {
                format!("{}:{}", path, current_xdg)
            };
            cmd.env("XDG_DATA_DIRS", new_xdg);
        }

        if let Some(gio) = gio_modules {
            let new_gio = if current_gio.is_empty() {
                gio.to_string()
            } else {
                format!("{}:{}", gio, current_gio)
            };
            cmd.env("GIO_EXTRA_MODULES", new_gio);
        }

        let _ = cmd.exec();
    }
}

fn main() {
    ensure_environment();
    let app = RelmApp::new("io.omacom.omawrite-rs");
    theme::init_custom_styles();

    let initial_path = std::env::args().nth(1).map(std::path::PathBuf::from);
    app.run::<AppModel>(initial_path);
}
