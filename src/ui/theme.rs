use std::fs;
use std::path::PathBuf;

pub fn init_custom_styles() {
    init_css();
    init_style_schemes();
}

fn init_css() {
    let provider = gtk4::CssProvider::new();
    provider.load_from_data(
        "
        /* Make the writing field, margins, and sidebar seamless with the window theme */
        textview,
        textview text,
        sourceview,
        sourceview text,
        .editor-canvas,
        .editor-canvas text {
            background-color: transparent;
            background: transparent;
            font-family: 'IBM Plex Mono', 'monospace';
            font-size: 15px;
            line-height: 1.5;
        }
        scrolledwindow,
        scrolledwindow > viewport,
        adw-clamp {
            background-color: transparent;
            background: transparent;
        }
        splitview > .sidebar,
        overlaysplitview > .sidebar,
        .outline-sidebar-panel {
            background-color: transparent;
            background: transparent;
            font-family: 'IBM Plex Mono', 'monospace';
            border-left: 1px solid alpha(currentColor, 0.07);
            border-right: 1px solid alpha(currentColor, 0.07);
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
        let dark_scheme = r##"<?xml version="1.0" encoding="UTF-8"?>
<style-scheme id="omawrite-dark" _name="Omawrite Dark" version="1.0" parent-scheme="Adwaita-dark">
  <color name="transparent" value="#00000000"/>
  <style name="text" background="transparent"/>
  <style name="current-line" background="transparent"/>
  <style name="def:heading" scale="1.3" bold="true"/>
  <style name="markdown:header" scale="1.3" bold="true"/>
</style-scheme>
"##;
        let light_scheme = r##"<?xml version="1.0" encoding="UTF-8"?>
<style-scheme id="omawrite-light" _name="Omawrite Light" version="1.0" parent-scheme="Adwaita">
  <color name="transparent" value="#00000000"/>
  <style name="text" background="transparent"/>
  <style name="current-line" background="transparent"/>
  <style name="def:heading" scale="1.3" bold="true"/>
  <style name="markdown:header" scale="1.3" bold="true"/>
</style-scheme>
"##;

        let _ = fs::write(styles_dir.join("omawrite-dark.xml"), dark_scheme);
        let _ = fs::write(styles_dir.join("omawrite-light.xml"), light_scheme);

        let manager = sourceview5::StyleSchemeManager::default();
        if let Some(dir_str) = styles_dir.to_str() {
            manager.prepend_search_path(dir_str);
        }
    }
}
