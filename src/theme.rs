pub fn init_custom_styles() {
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
