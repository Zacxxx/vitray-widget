use gtk4::prelude::*;
use gtk4::{Window, Box, Orientation, Label, Switch, ComboBoxText, Button};
use crate::settings::{Settings, Theme};

pub fn show_settings_window(parent: &impl IsA<Window>) {
    let window = Window::new();
    window.set_title(Some("Vitray Settings"));
    window.set_transient_for(Some(parent));
    window.set_modal(true);
    window.set_default_size(300, 400);

    let main_box = Box::new(Orientation::Vertical, 10);
    main_box.set_margin_top(20);
    main_box.set_margin_bottom(20);
    main_box.set_margin_start(20);
    main_box.set_margin_end(20);

    let settings = Settings::load();

    // Theme
    let theme_box = Box::new(Orientation::Horizontal, 10);
    theme_box.append(&Label::new(Some("Theme")));
    let theme_combo = ComboBoxText::new();
    theme_combo.append_text("Dark");
    theme_combo.append_text("Light");
    theme_combo.append_text("Solarized");
    theme_combo.append_text("Tokyo");
    // Set active based on settings.theme
    theme_combo.set_active(Some(match settings.theme {
        Theme::Dark => 0,
        Theme::Light => 1,
        Theme::Solarized => 2,
        Theme::Tokyo => 3,
    }));
    theme_box.append(&theme_combo);
    main_box.append(&theme_box);

    // Toggles
    let cpu_box = create_toggle("Show CPU", settings.show_cpu);
    main_box.append(&cpu_box);
    
    let gpu_box = create_toggle("Show GPU", settings.show_gpu);
    main_box.append(&gpu_box);

    let ram_box = create_toggle("Show RAM", settings.show_ram);
    main_box.append(&ram_box);

    let net_box = create_toggle("Show Network", settings.show_network);
    main_box.append(&net_box);

    let auto_start_box = create_toggle("Launch at Start", settings.launch_at_start);
    main_box.append(&auto_start_box);

    // Save Button
    let save_btn = Button::with_label("Save");
    let window_clone = window.clone();
    
    // Clone widgets for closure
    let theme_combo = theme_combo.clone();
    let cpu_switch = cpu_box.last_child().unwrap().downcast::<Switch>().unwrap();
    let gpu_switch = gpu_box.last_child().unwrap().downcast::<Switch>().unwrap();
    let ram_switch = ram_box.last_child().unwrap().downcast::<Switch>().unwrap();
    let net_switch = net_box.last_child().unwrap().downcast::<Switch>().unwrap();
    let auto_start_switch = auto_start_box.last_child().unwrap().downcast::<Switch>().unwrap();

    save_btn.connect_clicked(move |_| {
        let mut new_settings = Settings::load();
        
        // Theme
        new_settings.theme = match theme_combo.active().unwrap_or(0) {
            0 => Theme::Dark,
            1 => Theme::Light,
            2 => Theme::Solarized,
            3 => Theme::Tokyo,
            _ => Theme::Dark,
        };

        // Toggles
        new_settings.show_cpu = cpu_switch.is_active();
        new_settings.show_gpu = gpu_switch.is_active();
        new_settings.show_ram = ram_switch.is_active();
        new_settings.show_network = net_switch.is_active();
        
        // Auto-launch
        let launch = auto_start_switch.is_active();
        new_settings.set_auto_launch(launch); // This saves internally

        // Save other settings
        new_settings.save();
        
        println!("Settings saved");
        window_clone.close();
    });
    main_box.append(&save_btn);

    window.set_child(Some(&main_box));
    window.present();
}

fn create_toggle(label: &str, active: bool) -> Box {
    let hbox = Box::new(Orientation::Horizontal, 10);
    let lbl = Label::new(Some(label));
    lbl.set_hexpand(true);
    lbl.set_halign(gtk4::Align::Start);
    let switch = Switch::new();
    switch.set_active(active);
    hbox.append(&lbl);
    hbox.append(&switch);
    hbox
}
