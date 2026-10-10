use gtk4::prelude::*;
use sourceview5::prelude::*;

pub fn create_markdown_buffer(is_dark: bool) -> sourceview5::Buffer {
    let buffer = sourceview5::Buffer::new(None);

    let lang_manager = sourceview5::LanguageManager::default();
    if let Some(markdown_lang) = lang_manager.language("markdown") {
        buffer.set_language(Some(&markdown_lang));
        buffer.set_highlight_syntax(true);
    }

    apply_style_scheme(&buffer, is_dark);
    buffer
}

pub fn apply_style_scheme(buffer: &sourceview5::Buffer, is_dark: bool) {
    let scheme_manager = sourceview5::StyleSchemeManager::default();
    let scheme_id = if is_dark { "Adwaita-dark" } else { "Adwaita" };

    if let Some(scheme) = scheme_manager.scheme(scheme_id) {
        buffer.set_style_scheme(Some(&scheme));
    } else if let Some(fallback) = scheme_manager.scheme("classic") {
        buffer.set_style_scheme(Some(&fallback));
    }
}

pub fn configure_editor_view(view: &sourceview5::View) {
    view.set_monospace(true);
    view.set_wrap_mode(gtk4::WrapMode::Word);
    view.set_show_line_numbers(false);
    view.set_show_line_marks(false);
    view.set_highlight_current_line(false);
    view.set_auto_indent(true);
    view.set_indent_width(4);
    view.set_tab_width(4);
    view.set_insert_spaces_instead_of_tabs(true);

    view.set_top_margin(32);
    view.set_bottom_margin(64);
    view.set_left_margin(16);
    view.set_right_margin(16);

    view.add_css_class("editor-canvas");
}

pub fn format_bold(buffer: &sourceview5::Buffer) {
    wrap_selection(buffer, "**", "**", "bold text");
}

pub fn format_italic(buffer: &sourceview5::Buffer) {
    wrap_selection(buffer, "*", "*", "italic text");
}

pub fn format_link(buffer: &sourceview5::Buffer, clipboard_url: Option<&str>) {
    let url = clipboard_url.unwrap_or("url");
    let suffix = format!("]({})", url);
    wrap_selection(buffer, "[", &suffix, "link text");
}

fn wrap_selection(
    buffer: &sourceview5::Buffer,
    prefix: &str,
    suffix: &str,
    placeholder: &str,
) {
    buffer.begin_user_action();

    if let Some((mut start, end)) = buffer.selection_bounds() {
        // end_mark has right gravity (false) so inserting before it moves the mark forward
        let end_mark = buffer.create_mark(None, &end, false);
        let start_mark = buffer.create_mark(None, &start, true);

        buffer.insert(&mut start, prefix);

        let mut updated_end = buffer.iter_at_mark(&end_mark);
        buffer.insert(&mut updated_end, suffix);

        let sel_start = buffer.iter_at_mark(&start_mark);
        let sel_end = buffer.iter_at_mark(&end_mark);
        buffer.select_range(&sel_end, &sel_start);

        buffer.delete_mark(&start_mark);
        buffer.delete_mark(&end_mark);
    } else {
        let insert_mark = buffer.get_insert();
        let mut iter = buffer.iter_at_mark(&insert_mark);

        let start_mark = buffer.create_mark(None, &iter, true);
        buffer.insert(&mut iter, prefix);

        let placeholder_mark = buffer.create_mark(None, &iter, true);
        buffer.insert(&mut iter, placeholder);

        let end_mark = buffer.create_mark(None, &iter, false);
        buffer.insert(&mut iter, suffix);

        let sel_start = buffer.iter_at_mark(&placeholder_mark);
        let sel_end = buffer.iter_at_mark(&end_mark);
        buffer.select_range(&sel_end, &sel_start);

        buffer.delete_mark(&start_mark);
        buffer.delete_mark(&placeholder_mark);
        buffer.delete_mark(&end_mark);
    }

    buffer.end_user_action();
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_formatting() {
        let _ = gtk4::init();
        let buffer = sourceview5::Buffer::new(None);

        // 1. Test bold
        buffer.set_text("Hello world");
        let start = buffer.iter_at_offset(6);
        let end = buffer.iter_at_offset(11);
        buffer.select_range(&end, &start);
        format_bold(&buffer);
        assert_eq!(
            buffer.text(&buffer.start_iter(), &buffer.end_iter(), false),
            "Hello **world**"
        );

        // 2. Test italic
        buffer.set_text("Hello world");
        let start = buffer.iter_at_offset(0);
        let end = buffer.iter_at_offset(5);
        buffer.select_range(&end, &start);
        format_italic(&buffer);
        assert_eq!(
            buffer.text(&buffer.start_iter(), &buffer.end_iter(), false),
            "*Hello* world"
        );
    }
}
