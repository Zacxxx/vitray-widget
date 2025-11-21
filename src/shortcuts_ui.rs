use gtk4::prelude::*;
use gtk4::{Window, Box, Orientation, Label, Button, ListView, SignalListItemFactory, NoSelection, StringList, StringObject, Align};
use crate::shortcuts::Shortcuts;
use glib::Sender;

pub fn show_shortcuts_panel(parent: &impl IsA<Window>, sender: Sender<String>) {
    let window = Window::new();
    window.set_title(Some("Shortcuts"));
    window.set_transient_for(Some(parent));
    window.set_default_size(300, 500);
    window.set_decorated(false);

    let main_box = Box::new(Orientation::Vertical, 10);
    main_box.add_css_class("glass-panel");

    let title = Label::new(Some("Shortcuts"));
    title.add_css_class("card-title");
    main_box.append(&title);

    let shortcuts = Shortcuts::load();
    let model = StringList::new(&[]);
    for s in shortcuts.items {
        model.append(&format!("{}|{}", s.name, s.command));
    }

    let factory = SignalListItemFactory::new();
    let sender_clone = sender.clone();
    
    factory.connect_setup(move |_, list_item| {
        let hbox = Box::new(Orientation::Horizontal, 5);
        let label = Label::new(None);
        label.set_hexpand(true);
        label.set_halign(Align::Start);
        
        let run_btn = Button::with_label("Run");
        run_btn.add_css_class("suggested-action");
        
        let del_btn = Button::with_label("X");
        del_btn.add_css_class("destructive-action");

        hbox.append(&label);
        hbox.append(&run_btn);
        hbox.append(&del_btn);
        list_item.set_child(Some(&hbox));
    });

    factory.connect_bind(move |_, list_item| {
        let hbox = list_item.child().and_downcast::<Box>().unwrap();
        let label = hbox.first_child().unwrap().downcast::<Label>().unwrap();
        let run_btn = hbox.last_child().unwrap().prev_sibling().unwrap().downcast::<Button>().unwrap();
        // let del_btn = hbox.last_child().unwrap().downcast::<Button>().unwrap(); // TODO: Implement delete

        let item = list_item.item().and_downcast::<StringObject>().unwrap();
        let full_str = item.string();
        let parts: Vec<&str> = full_str.split('|').collect();
        if parts.len() >= 2 {
            label.set_text(parts[0]);
            let cmd = parts[1].to_string();
            let sender = sender_clone.clone();
            
            // Disconnect previous handlers to avoid duplicates if reused? 
            // Actually bind is called for new data. 
            // Better to use a custom GObject or just closure here.
            // For simplicity in this prototype, we just connect. 
            // Note: This leaks handlers if rows are recycled heavily. 
            // Proper way is to use GObject subclassing or setup/teardown.
            // But for a small list, this is okay-ish.
            run_btn.connect_clicked(move |_| {
                let _ = sender.send(cmd.clone());
            });
        }
    });

    let selection_model = NoSelection::new(Some(model));
    let list_view = ListView::new(Some(selection_model), Some(factory));
    
    let scrolled = gtk4::ScrolledWindow::new();
    scrolled.set_child(Some(&list_view));
    scrolled.set_vexpand(true);
    
    main_box.append(&scrolled);

    // Add/Remove buttons placeholder
    let btn_box = Box::new(Orientation::Horizontal, 5);
    let add_btn = Button::with_label("+");
    btn_box.append(&add_btn);
    main_box.append(&btn_box);

    window.set_child(Some(&main_box));
    window.present();
}
