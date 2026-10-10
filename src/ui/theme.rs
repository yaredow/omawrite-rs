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
