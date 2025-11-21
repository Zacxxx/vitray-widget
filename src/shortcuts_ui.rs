use glib::Sender;
use gtk4::prelude::*;
use gtk4::{
    Align, ApplicationWindow, Box, Button, Dialog, Entry, Label, ListBox, ListBoxRow, Orientation,
    Overlay, Popover, Revealer, RevealerTransitionType,
};
use std::{cell::RefCell, rc::Rc};

use crate::shortcuts::{Shortcut, Shortcuts};

#[derive(Clone)]
pub struct ShortcutsPanel {
    pub revealer: Revealer,
    list: ListBox,
    data: Rc<RefCell<Shortcuts>>,
    parent: ApplicationWindow,
    sender: Sender<String>,
}

impl ShortcutsPanel {
    pub fn new(parent: &ApplicationWindow, sender: Sender<String>) -> Self {
        let revealer = Revealer::new();
        revealer.set_transition_type(RevealerTransitionType::SlideLeft);
        revealer.set_reveal_child(true);

        let column = Box::new(Orientation::Vertical, 10);
        column.add_css_class("shortcuts-panel");
        column.set_margin_top(12);
        column.set_margin_bottom(12);
        column.set_margin_end(12);
        column.set_margin_start(6);

        let title = Label::new(Some("Shortcuts"));
        title.add_css_class("panel-title");
        title.set_halign(Align::Start);
        column.append(&title);

        let list = ListBox::new();
        list.add_css_class("shortcut-list");
        list.set_selection_mode(gtk4::SelectionMode::None);
        column.append(&list);

        let add_btn = Button::with_label("Create Shortcut");
        add_btn.add_css_class("pill-btn");
        column.append(&add_btn);

        revealer.set_child(Some(&column));

        let data = Rc::new(RefCell::new(Shortcuts::load()));
        let panel = Self {
            revealer,
            list,
            data,
            parent: parent.clone(),
            sender,
        };

        {
            let panel_clone = panel.clone();
            add_btn.connect_clicked(move |_| open_editor(&panel_clone, None));
        }

        panel.refresh();
        panel
    }

    pub fn set_revealed(&self, show: bool) {
        self.revealer.set_reveal_child(show);
    }

    pub fn toggle(&self) {
        self.revealer
            .set_reveal_child(!self.revealer.reveals_child());
    }

    pub fn refresh(&self) {
        while let Some(child) = self.list.first_child() {
            self.list.remove(&child);
        }

        for shortcut in self.data.borrow().items.clone() {
            let row = build_row(self.clone(), shortcut);
            self.list.append(&row);
        }
    }

    pub fn run_shortcut(&self, command: String) {
        let _ = self.sender.send(command);
    }
}

fn build_row(panel: ShortcutsPanel, shortcut: Shortcut) -> ListBoxRow {
    let row = ListBoxRow::new();
    row.add_css_class("shortcut-row");

    let overlay = Overlay::new();
    let content = Box::new(Orientation::Vertical, 4);

    let title = Label::new(Some(&shortcut.name));
    title.add_css_class("shortcut-title");
    title.set_halign(Align::Start);

    let command = Label::new(Some(&shortcut.command));
    command.add_css_class("shortcut-command");
    command.set_halign(Align::Start);
    command.set_wrap(true);

    let actions = Box::new(Orientation::Horizontal, 6);
    actions.set_halign(Align::End);

    let use_btn = Button::with_label("Use");
    use_btn.add_css_class("pill-btn");

    let edit_btn = Button::from_icon_name("document-edit-symbolic");
    edit_btn.add_css_class("icon-btn");
    edit_btn.set_tooltip_text(Some("Edit shortcut"));

    let delete_btn = Button::from_icon_name("window-close-symbolic");
    delete_btn.add_css_class("icon-btn");
    delete_btn.set_tooltip_text(Some("Delete shortcut"));

    actions.append(&edit_btn);
    actions.append(&use_btn);
    actions.append(&delete_btn);

    content.append(&title);
    content.append(&command);
    content.append(&actions);

    overlay.set_child(Some(&content));
    row.set_child(Some(&overlay));

    let shortcut_name = shortcut.name.clone();
    let shortcut_command = shortcut.command.clone();
    {
        let panel_clone = panel.clone();
        use_btn.connect_clicked(move |_| {
            panel_clone.run_shortcut(shortcut_command.clone());
        });
    }

    {
        let panel_clone = panel.clone();
        edit_btn.connect_clicked(move |_| {
            open_editor(&panel_clone, Some(shortcut.clone()));
        });
    }

    {
        let panel_clone = panel.clone();
        delete_btn.connect_clicked(move |btn| {
            let popover = build_delete_popover(&panel_clone, shortcut_name.clone(), btn);
            popover.popup();
        });
    }

    row
}

fn build_delete_popover(panel: &ShortcutsPanel, name: String, anchor: &Button) -> Popover {
    let popover = Popover::builder().has_arrow(true).build();
    popover.set_parent(anchor);

    let column = Box::new(Orientation::Vertical, 6);
    column.set_margin_top(8);
    column.set_margin_bottom(8);
    column.set_margin_start(8);
    column.set_margin_end(8);

    let prompt = Label::new(Some("Delete shortcut?"));
    prompt.add_css_class("popover-label");
    column.append(&prompt);

    let actions = Box::new(Orientation::Horizontal, 6);
    let cancel = Button::with_label("Cancel");
    let confirm = Button::with_label("Delete");
    confirm.add_css_class("danger");
    actions.append(&cancel);
    actions.append(&confirm);
    column.append(&actions);

    {
        let pop = popover.clone();
        cancel.connect_clicked(move |_| pop.popdown());
    }

    {
        let pop = popover.clone();
        let panel_clone = panel.clone();
        confirm.connect_clicked(move |_| {
            panel_clone.data.borrow_mut().remove_by_name(&name);
            panel_clone.refresh();
            pop.popdown();
        });
    }

    popover.set_child(Some(&column));
    popover
}

fn open_editor(panel: &ShortcutsPanel, existing: Option<Shortcut>) {
    let dialog = Dialog::builder()
        .transient_for(&panel.parent)
        .modal(true)
        .title(
            existing
                .as_ref()
                .map(|_| "Edit Shortcut")
                .unwrap_or("Add Shortcut"),
        )
        .build();
    dialog.set_default_size(360, 180);

    let area = dialog.content_area();
    area.set_spacing(8);
    area.set_margin_top(12);
    area.set_margin_bottom(12);
    area.set_margin_start(12);
    area.set_margin_end(12);

    let name_entry = Entry::new();
    name_entry.set_placeholder_text(Some("Shortcut name"));
    let cmd_entry = Entry::new();
    cmd_entry.set_placeholder_text(Some("Command to run"));

    if let Some(ref shortcut) = existing {
        name_entry.set_text(&shortcut.name);
        cmd_entry.set_text(&shortcut.command);
    }

    area.append(&Label::new(Some("Name")));
    area.append(&name_entry);
    area.append(&Label::new(Some("Command")));
    area.append(&cmd_entry);

    let actions = Box::new(Orientation::Horizontal, 8);
    actions.set_halign(Align::End);
    let cancel = Button::with_label("Cancel");
    let save = Button::with_label("Save");
    save.add_css_class("pill-btn");
    actions.append(&cancel);
    actions.append(&save);
    area.append(&actions);

    {
        let dialog_clone = dialog.clone();
        cancel.connect_clicked(move |_| dialog_clone.close());
    }

    {
        let dialog_clone = dialog.clone();
        let panel_clone = panel.clone();
        let original = existing.map(|s| s.name);
        save.connect_clicked(move |_| {
            let name = name_entry.text().trim().to_string();
            let command = cmd_entry.text().trim().to_string();
            if name.is_empty() || command.is_empty() {
                dialog_clone.close();
                return;
            }

            if let Some(old) = &original {
                panel_clone
                    .data
                    .borrow_mut()
                    .rename(old, name.clone(), command.clone());
            } else {
                panel_clone
                    .data
                    .borrow_mut()
                    .upsert(name.clone(), command.clone());
            }
            panel_clone.refresh();
            dialog_clone.close();
        });
    }

    dialog.show();
}
