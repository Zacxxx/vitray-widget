use gtk4::prelude::*;
use gtk4::{
    Align, Box, Button, ComboBoxText, Label, Notebook, Orientation, Switch, TextView, Window,
};
use std::{cell::RefCell, rc::Rc};

use crate::settings::{MonitorStyle, Settings, Theme};

pub fn show_settings_window(
    parent: &impl IsA<Window>,
    settings: Rc<RefCell<Settings>>,
    on_save: impl Fn(Settings) + 'static,
) {
    let window = Window::new();
    window.set_title(Some("Vitray Settings"));
    window.set_transient_for(Some(parent));
    window.set_modal(true);
    window.set_default_size(420, 520);
    window.add_css_class("glass-panel");

    let notebook = Notebook::new();
    notebook.add_css_class("settings-notebook");

    let main_box = Box::new(Orientation::Vertical, 12);
    main_box.set_margin_top(16);
    main_box.set_margin_bottom(16);
    main_box.set_margin_start(16);
    main_box.set_margin_end(16);
    main_box.set_spacing(12);

    let settings_snapshot = settings.borrow().clone();

    // Theme
    let theme_box = Box::new(Orientation::Horizontal, 10);
    theme_box.append(&Label::new(Some("Theme")));
    let theme_combo = ComboBoxText::new();
    theme_combo.append_text("Dark");
    theme_combo.append_text("Light");
    theme_combo.append_text("Solarized");
    theme_combo.append_text("Tokyo");
    theme_combo.set_active(Some(match settings_snapshot.theme {
        Theme::Dark => 0,
        Theme::Light => 1,
        Theme::Solarized => 2,
        Theme::Tokyo => 3,
    }));
    theme_box.append(&theme_combo);
    main_box.append(&theme_box);

    // Monitor style
    let style_box = Box::new(Orientation::Horizontal, 10);
    style_box.append(&Label::new(Some("Monitoring style")));
    let style_combo = ComboBoxText::new();
    style_combo.append_text("Just numbers");
    style_combo.append_text("Bar");
    style_combo.append_text("Chart");
    style_combo.set_active(Some(match settings_snapshot.monitor_style {
        MonitorStyle::Text => 0,
        MonitorStyle::Bar => 1,
        MonitorStyle::Chart => 2,
    }));
    style_box.append(&style_combo);
    main_box.append(&style_box);

    // Sections
    let terminal_toggle = create_toggle("Show terminal", settings_snapshot.show_terminal);
    let monitoring_toggle = create_toggle("Show monitoring", settings_snapshot.show_monitoring);
    let shortcuts_toggle = create_toggle(
        "Show shortcuts panel",
        settings_snapshot.show_shortcuts_panel,
    );
    let cpu_box = create_toggle("Show CPU", settings_snapshot.show_cpu);
    let gpu_box = create_toggle("Show GPU", settings_snapshot.show_gpu);
    let ram_box = create_toggle("Show RAM", settings_snapshot.show_ram);
    let net_box = create_toggle("Show Network", settings_snapshot.show_network);
    main_box.append(&terminal_toggle.0);
    main_box.append(&monitoring_toggle.0);
    main_box.append(&shortcuts_toggle.0);
    main_box.append(&cpu_box.0);
    main_box.append(&gpu_box.0);
    main_box.append(&ram_box.0);
    main_box.append(&net_box.0);

    let auto_start_box = create_toggle("Launch at start", settings_snapshot.launch_at_start);
    let lock_place_box = create_toggle("Lock position", settings_snapshot.lock_in_place);
    let lock_size_box = create_toggle("Lock size", settings_snapshot.lock_size);
    main_box.append(&auto_start_box.0);
    main_box.append(&lock_place_box.0);
    main_box.append(&lock_size_box.0);

    // Save Button
    let save_btn = Button::with_label("Save");
    save_btn.add_css_class("pill-btn");
    let window_clone = window.clone();

    save_btn.connect_clicked(move |_| {
        let mut new_settings = settings_snapshot.clone();

        // Theme
        new_settings.theme = match theme_combo.active().unwrap_or(0) {
            0 => Theme::Dark,
            1 => Theme::Light,
            2 => Theme::Solarized,
            3 => Theme::Tokyo,
            _ => Theme::Dark,
        };

        new_settings.monitor_style = match style_combo.active().unwrap_or(0) {
            1 => MonitorStyle::Bar,
            2 => MonitorStyle::Chart,
            _ => MonitorStyle::Text,
        };

        new_settings.show_terminal = terminal_toggle.1.is_active();
        new_settings.show_monitoring = monitoring_toggle.1.is_active();
        new_settings.show_shortcuts_panel = shortcuts_toggle.1.is_active();
        new_settings.show_cpu = cpu_box.1.is_active();
        new_settings.show_gpu = gpu_box.1.is_active();
        new_settings.show_ram = ram_box.1.is_active();
        new_settings.show_network = net_box.1.is_active();

        let launch = auto_start_box.1.is_active();
        new_settings.set_auto_launch(launch);

        new_settings.lock_in_place = lock_place_box.1.is_active();
        new_settings.lock_size = lock_size_box.1.is_active();

        settings.replace(new_settings.clone());
        new_settings.save();
        on_save(new_settings);
        window_clone.close();
    });
    main_box.append(&save_btn);

    notebook.append_page(&main_box, Some(&Label::new(Some("Controls"))));

    // Help tab
    let help_box = Box::new(Orientation::Vertical, 10);
    help_box.set_margin_top(16);
    help_box.set_margin_bottom(16);
    help_box.set_margin_start(16);
    help_box.set_margin_end(16);

    let help_text = TextView::new();
    help_text.set_editable(false);
    help_text.set_wrap_mode(gtk4::WrapMode::Word);
    help_text.add_css_class("help-text");
    help_text.buffer().set_text(
        "Tips:\n\
         - Right click the widget for quick actions.\n\
         - Use the header buttons to minimize, maximize, or close.\n\
         - Create shortcuts: vitray --shortcut \"command\" \"name\".\n\
         - Run shortcuts: vitray <name>.\n\
         - Toggle themes here or via CSS in src/style.css.",
    );
    help_box.append(&help_text);
    notebook.append_page(&help_box, Some(&Label::new(Some("Help"))));

    window.set_child(Some(&notebook));
    window.present();
}

fn create_toggle(label: &str, active: bool) -> (Box, Switch) {
    let hbox = Box::new(Orientation::Horizontal, 10);
    let lbl = Label::new(Some(label));
    lbl.set_hexpand(true);
    lbl.set_halign(Align::Start);
    let switch = Switch::new();
    switch.set_active(active);
    hbox.append(&lbl);
    hbox.append(&switch);
    (hbox, switch)
}
