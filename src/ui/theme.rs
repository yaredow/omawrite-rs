use relm4::adw;
use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;

thread_local! {
    static EDITOR_THEME_PROVIDER: RefCell<Option<gtk4::CssProvider>> = const { RefCell::new(None) };
}

pub fn init_custom_styles() {
    init_css();
    init_style_schemes();
    let is_dark = adw::StyleManager::default().is_dark();
    apply_editor_theme(is_dark);
}

pub fn apply_editor_theme(is_dark: bool) {
    EDITOR_THEME_PROVIDER.with(|cell| {
        let mut opt = cell.borrow_mut();
        let provider = opt.get_or_insert_with(|| {
            let p = gtk4::CssProvider::new();
            if let Some(display) = gtk4::gdk::Display::default() {
                gtk4::style_context_add_provider_for_display(
                    &display,
                    &p,
                    gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION + 5,
                );
            }
            p
        });

        let bg = if is_dark { "#1d1d20" } else { "#ffffff" };
        provider.load_from_data(&format!(
            "
            .editor-scroll-area,
            .editor-scroll-area > viewport,
            .editor-scroll-area adw-clamp,
            .editor-scroll-area textview,
            .editor-scroll-area textview text,
            .editor-scroll-area sourceview,
            .editor-scroll-area sourceview text,
            .editor-canvas,
            .editor-canvas text {{
                background-color: {bg};
                background: {bg};
            }}
            "
        ));
    });
}

fn init_css() {
    let provider = gtk4::CssProvider::new();
    provider.load_from_data(
        "
        textview {
            font-family: 'IBM Plex Mono', 'monospace';
            font-size: 15px;
            line-height: 1.5;
        }
        .footer-bar {
            padding: 6px 18px;
            border-top: 1px solid alpha(currentColor, 0.07);
            font-size: 12px;
        }
        .status-label {
            opacity: 0.7;
            font-family: 'IBM Plex Mono', 'monospace';
        }
        .word-count {
            opacity: 0.7;
            font-family: 'IBM Plex Mono', 'monospace';
            font-feature-settings: 'tnum';
        }
        .outline-sidebar-panel {
            background-color: alpha(currentColor, 0.025);
            font-family: 'IBM Plex Mono', 'monospace';
        }
        .outline-title-label {
            font-size: 11px;
            font-weight: 700;
            letter-spacing: 0.08em;
            opacity: 0.55;
        }
        .outline-listbox {
            background: transparent;
            padding: 4px;
        }
        .outline-row {
            border-radius: 6px;
            padding: 1px 2px;
            transition: background 120ms ease;
        }
        .outline-row:hover {
            background-color: alpha(currentColor, 0.06);
        }
        .active-outline-row {
            background-color: alpha(@accent_color, 0.15);
            color: @accent_color;
        }
        .outline-h1 {
            font-weight: 600;
            font-size: 13px;
        }
        .outline-hn {
            font-size: 12.5px;
            opacity: 0.85;
        }
        .outline-empty-heading {
            font-size: 13px;
            font-weight: 600;
            opacity: 0.7;
        }
        ",
    );

    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

fn init_style_schemes() {
    let styles_dir: PathBuf = glib::user_data_dir().join("omawrite-rs").join("styles");
    if fs::create_dir_all(&styles_dir).is_ok() {
        let dark_scheme = r#"<?xml version="1.0" encoding="UTF-8"?>
<style-scheme id="omawrite-dark" _name="Omawrite Dark" version="1.0" parent-scheme="Adwaita-dark">
  <style name="def:heading" scale="1.3" bold="true"/>
  <style name="markdown:header" scale="1.3" bold="true"/>
</style-scheme>
"#;
        let light_scheme = r#"<?xml version="1.0" encoding="UTF-8"?>
<style-scheme id="omawrite-light" _name="Omawrite Light" version="1.0" parent-scheme="Adwaita">
  <style name="def:heading" scale="1.3" bold="true"/>
  <style name="markdown:header" scale="1.3" bold="true"/>
</style-scheme>
"#;

        let _ = fs::write(styles_dir.join("omawrite-dark.xml"), dark_scheme);
        let _ = fs::write(styles_dir.join("omawrite-light.xml"), light_scheme);

        let manager = sourceview5::StyleSchemeManager::default();
        if let Some(dir_str) = styles_dir.to_str() {
            manager.prepend_search_path(dir_str);
        }
    }
}
