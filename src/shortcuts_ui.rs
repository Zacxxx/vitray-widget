use async_channel::Sender;
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

        let search_entry = gtk4::SearchEntry::new();
        search_entry.set_placeholder_text(Some("Search shortcuts..."));
        column.append(&search_entry);

        let list = ListBox::new();
        list.add_css_class("shortcut-list");
        list.set_selection_mode(gtk4::SelectionMode::None);
        
        // Filter list based on search
        let list_clone = list.clone();
        search_entry.connect_search_changed(move |entry| {
            let query = entry.text().to_lowercase();
            let mut child = list_clone.first_child();
            while let Some(row) = child {
                if let Ok(row_widget) = row.clone().downcast::<ListBoxRow>() {
                     if let Some(label) = row_widget.child().and_then(|c| c.downcast::<Overlay>().ok())
                        .and_then(|o| o.child())
                        .and_then(|c| c.downcast::<Box>().ok())
                        .and_then(|b| b.first_child())
                        .and_then(|c| c.downcast::<Label>().ok()) {
                            let text = label.text().to_lowercase();
                            row_widget.set_visible(text.contains(&query));
                     }
                }
                child = row.next_sibling();
            }
        });

        column.append(&list);

    let add_btn = Button::with_label("+ New Shortcut");
    add_btn.add_css_class("pill-btn");
    add_btn.add_css_class("suggested-action");
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
        let _ = self.sender.try_send(command);
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

    let command_box = Box::new(Orientation::Horizontal, 4);
    let cmd_chip = Label::new(Some(&shortcut.command));
    cmd_chip.add_css_class("command-chip");
    cmd_chip.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    command_box.append(&cmd_chip);
    
    let copy_btn = Button::from_icon_name("edit-copy-symbolic");
    copy_btn.add_css_class("flat");
    copy_btn.add_css_class("small-icon");
    copy_btn.set_tooltip_text(Some("Copy command"));
    
    let cmd_text = shortcut.command.clone();
    copy_btn.connect_clicked(move |_| {
        if let Some(display) = gtk4::gdk::Display::default() {
            display.clipboard().set(&cmd_text);
        }
    });
    command_box.append(&copy_btn);
    
    command_box.set_halign(Align::Start);

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
    content.append(&title);
    content.append(&command_box);
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
        let panel_clone = panel;
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
                .map_or("Add Shortcut", |_| "Edit Shortcut"),
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
                name_entry.add_css_class("error");
                cmd_entry.add_css_class("error");
                return;
            }

            let result = if let Some(old) = &original {
                panel_clone
                    .data
                    .borrow_mut()
                    .rename(old, name, command)
            } else {
                panel_clone
                    .data
                    .borrow_mut()
                    .add(&name, command)
            };
            
            match result {
                Ok(()) => {
                    panel_clone.refresh();
                    dialog_clone.close();
                }
                Err(e) => {
                    // Show error in placeholder or tooltip
                    name_entry.add_css_class("error");
                    name_entry.set_tooltip_text(Some(&e));
                }
            }
        });
    }

    dialog.show();
}
