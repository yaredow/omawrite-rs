use gtk4::prelude::*;
use relm4::prelude::*;

use crate::core::config::SidebarPosition;
use crate::core::outline::HeadingItem;

pub struct OutlineModel {
    pub headings: Vec<HeadingItem>,
    pub active_line: Option<u32>,
    pub position: SidebarPosition,
    pub list_box: gtk4::ListBox,
}

#[derive(Clone, Debug)]
pub enum OutlineInput {
    SetHeadings(Vec<HeadingItem>),
    SetActiveLine(u32),
    RowActivated(usize),
    TogglePosition,
    Close,
}

#[derive(Clone, Debug)]
pub enum OutlineOutput {
    JumpToLine(u32),
    PositionChanged(SidebarPosition),
    CloseRequested,
}

#[relm4::component(pub)]
impl SimpleComponent for OutlineModel {
    type Init = SidebarPosition;
    type Input = OutlineInput;
    type Output = OutlineOutput;

    view! {
        gtk4::Box {
            set_orientation: gtk4::Orientation::Vertical,
            set_width_request: 230,
            add_css_class: "outline-sidebar-panel",

            // Sidebar Header Bar
            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 4,
                set_margin_start: 12,
                set_margin_end: 6,
                set_margin_top: 8,
                set_margin_bottom: 8,

                gtk4::Label {
                    set_label: "OUTLINE",
                    set_hexpand: true,
                    set_xalign: 0.0,
                    add_css_class: "outline-title-label",
                },

                gtk4::Button {
                    #[watch]
                    set_icon_name: match model.position {
                        SidebarPosition::Right => "go-previous-symbolic",
                        SidebarPosition::Left => "go-next-symbolic",
                    },
                    #[watch]
                    set_tooltip_text: match model.position {
                        SidebarPosition::Right => Some("Move to Left (Ctrl+Alt+O)"),
                        SidebarPosition::Left => Some("Move to Right (Ctrl+Alt+O)"),
                    },
                    add_css_class: "flat",
                    connect_clicked => OutlineInput::TogglePosition,
                },

                gtk4::Button {
                    set_icon_name: "window-close-symbolic",
                    set_tooltip_text: Some("Close (Ctrl+Shift+O)"),
                    add_css_class: "flat",
                    connect_clicked => OutlineInput::Close,
                },
            },

            gtk4::Separator {
                set_orientation: gtk4::Orientation::Horizontal,
            },

            // Empty state placeholder
            gtk4::Box {
                set_orientation: gtk4::Orientation::Vertical,
                set_spacing: 8,
                set_margin_start: 20,
                set_margin_end: 20,
                set_margin_top: 48,
                #[watch]
                set_visible: model.headings.is_empty(),
                set_valign: gtk4::Align::Center,

                gtk4::Image {
                    set_icon_name: Some("document-edit-symbolic"),
                    set_pixel_size: 28,
                    add_css_class: "dim-label",
                },

                gtk4::Label {
                    set_label: "No headings yet",
                    add_css_class: "outline-empty-heading",
                    set_justify: gtk4::Justification::Center,
                },

                gtk4::Label {
                    set_label: "Type # Heading to build your document structure.",
                    set_wrap: true,
                    add_css_class: "dim-label",
                    set_justify: gtk4::Justification::Center,
                },
            },

            // Scrolled ListBox
            gtk4::ScrolledWindow {
                set_hexpand: true,
                set_vexpand: true,
                #[watch]
                set_visible: !model.headings.is_empty(),

                #[local_ref]
                list_box -> gtk4::ListBox {
                    set_selection_mode: gtk4::SelectionMode::None,
                    add_css_class: "outline-listbox",
                },
            },
        }
    }

    fn init(
        init_position: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let list_box = gtk4::ListBox::new();
        list_box.set_selection_mode(gtk4::SelectionMode::None);

        let sender_clone = sender.clone();
        list_box.connect_row_activated(move |_lb, row| {
            let idx = row.index();
            if idx >= 0 {
                sender_clone.input(OutlineInput::RowActivated(idx as usize));
            }
        });

        let model = OutlineModel {
            headings: Vec::new(),
            active_line: None,
            position: init_position,
            list_box: list_box.clone(),
        };

        let list_box = &model.list_box;
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            OutlineInput::SetHeadings(new_headings) => {
                self.headings = new_headings;
                self.rebuild_list(&sender);
            }
            OutlineInput::SetActiveLine(line) => {
                self.active_line = Some(line);
                self.update_active_row_styles();
            }
            OutlineInput::RowActivated(idx) => {
                if let Some(heading) = self.headings.get(idx) {
                    let _ = sender.output(OutlineOutput::JumpToLine(heading.line));
                }
            }
            OutlineInput::TogglePosition => {
                self.position = match self.position {
                    SidebarPosition::Left => SidebarPosition::Right,
                    SidebarPosition::Right => SidebarPosition::Left,
                };
                let _ = sender.output(OutlineOutput::PositionChanged(self.position));
            }
            OutlineInput::Close => {
                let _ = sender.output(OutlineOutput::CloseRequested);
            }
        }
    }
}

impl OutlineModel {
    fn is_heading_active(&self, idx: usize) -> bool {
        if let Some(cursor_line) = self.active_line {
            if let Some(h) = self.headings.get(idx) {
                if h.line <= cursor_line {
                    if let Some(next_h) = self.headings.get(idx + 1) {
                        return cursor_line < next_h.line;
                    }
                    return true;
                }
            }
        }
        false
    }

    fn rebuild_list(&self, _sender: &ComponentSender<Self>) {
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }

        for (idx, heading) in self.headings.iter().enumerate() {
            let row = gtk4::ListBoxRow::new();
            row.set_selectable(false);
            row.set_activatable(true);
            row.add_css_class("outline-row");

            let indent = match heading.level {
                1 => 6,
                2 => 18,
                3 => 30,
                4 => 42,
                5 => 54,
                _ => 66,
            };

            let hbox = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
            hbox.set_margin_start(indent);
            hbox.set_margin_end(6);
            hbox.set_margin_top(3);
            hbox.set_margin_bottom(3);

            let label = gtk4::Label::new(Some(&heading.title));
            label.set_xalign(0.0);
            label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
            label.set_hexpand(true);

            if heading.level == 1 {
                label.add_css_class("outline-h1");
            } else {
                label.add_css_class("outline-hn");
            }

            if self.is_heading_active(idx) {
                row.add_css_class("active-outline-row");
            }

            hbox.append(&label);
            row.set_child(Some(&hbox));
            self.list_box.append(&row);
        }
    }

    fn update_active_row_styles(&self) {
        let mut row_opt = self.list_box.first_child();
        let mut idx = 0;
        while let Some(widget) = row_opt {
            if let Ok(row) = widget.clone().downcast::<gtk4::ListBoxRow>() {
                if self.is_heading_active(idx) {
                    row.add_css_class("active-outline-row");
                } else {
                    row.remove_css_class("active-outline-row");
                }
            }
            row_opt = widget.next_sibling();
            idx += 1;
        }
    }
}
