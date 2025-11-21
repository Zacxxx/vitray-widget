use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box, Label, Orientation, CssProvider, 
    Grid, Notebook, GestureClick, glib
};
use crate::monitor::SystemMonitor;
use crate::terminal::create_terminal;
use crate::settings::Settings;
use crate::shortcuts::Shortcuts;
use vte4::TerminalExt;
use std::rc::Rc;
use std::cell::RefCell;

pub fn build_ui(app: &Application) {
    let _settings = Settings::load();
    let _shortcuts = Shortcuts::load();

    let provider = CssProvider::new();
    if std::path::Path::new("src/style.css").exists() {
        provider.load_from_path("src/style.css");
    } else {
        provider.load_from_path("/usr/share/vitray-widget/style.css");
    }
    
    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    let window = ApplicationWindow::new(app);
    window.set_title(Some("Vitray Widget"));
    window.set_default_size(400, 700);
    window.set_decorated(false);
    window.set_css_classes(&["glass-window"]);

    let main_box = Box::new(Orientation::Vertical, 0);
    main_box.add_css_class("glass-panel");

    // --- Terminal Section (Top) ---
    let notebook = Notebook::new();
    notebook.set_show_tabs(true);
    notebook.set_scrollable(true);
    notebook.add_css_class("terminal-notebook");

    // Initial Terminal Tab
    let terminal = create_terminal();
    let scrolled = gtk4::ScrolledWindow::new();
    scrolled.set_child(Some(&terminal));
    scrolled.set_vexpand(true);
    notebook.append_page(&scrolled, Some(&Label::new(Some("Terminal"))));

    // Add notebook to main box
    main_box.append(&notebook);

    // --- Separator ---
    let separator = gtk4::Separator::new(Orientation::Horizontal);
    separator.add_css_class("section-separator");
    main_box.append(&separator);

    // --- Monitoring Grid (Bottom) ---
    let grid = Grid::new();
    grid.set_column_spacing(10);
    grid.set_row_spacing(10);
    grid.set_margin_top(10);
    grid.set_margin_bottom(10);
    grid.set_margin_start(10);
    grid.set_margin_end(10);
    grid.add_css_class("monitoring-grid");

    // CPU Card
    let cpu_card = Box::new(Orientation::Vertical, 5);
    cpu_card.add_css_class("monitor-card");
    let cpu_label = Label::new(Some("CPU"));
    cpu_label.add_css_class("card-title");
    let cpu_value = Label::new(Some("0%"));
    cpu_value.add_css_class("card-value");
    cpu_card.append(&cpu_label);
    cpu_card.append(&cpu_value);
    grid.attach(&cpu_card, 0, 0, 1, 1);

    // GPU Card
    let gpu_card = Box::new(Orientation::Vertical, 5);
    gpu_card.add_css_class("monitor-card");
    let gpu_label = Label::new(Some("GPU"));
    gpu_label.add_css_class("card-title");
    let gpu_value = Label::new(Some("N/A"));
    gpu_value.add_css_class("card-value");
    gpu_card.append(&gpu_label);
    gpu_card.append(&gpu_value);
    grid.attach(&gpu_card, 1, 0, 1, 1);

    // RAM Card
    let ram_card = Box::new(Orientation::Vertical, 5);
    ram_card.add_css_class("monitor-card");
    let ram_label = Label::new(Some("RAM"));
    ram_label.add_css_class("card-title");
    let ram_value = Label::new(Some("0 GB"));
    ram_value.add_css_class("card-value");
    ram_card.append(&ram_label);
    ram_card.append(&ram_value);
    grid.attach(&ram_card, 0, 1, 1, 1);

    // Network Card
    let net_card = Box::new(Orientation::Vertical, 5);
    net_card.add_css_class("monitor-card");
    let net_label = Label::new(Some("NET"));
    net_label.add_css_class("card-title");
    let net_value = Label::new(Some("0 KB/s"));
    net_value.add_css_class("card-value");
    net_card.append(&net_label);
    net_card.append(&net_value);
    grid.attach(&net_card, 1, 1, 1, 1);

    main_box.append(&grid);

    window.set_child(Some(&main_box));

    // --- Shortcuts Channel ---
    #[allow(deprecated)]
    let (sender, receiver) = ::glib::MainContext::channel(::glib::Priority::DEFAULT);
    
    let terminal_clone = terminal.clone();
    receiver.attach(None, move |cmd: String| {
        let cmd_with_newline = format!("{}\n", cmd);
        terminal_clone.feed_child(cmd_with_newline.as_bytes());
        ::glib::ControlFlow::Continue
    });

    // --- Context Menu ---
    let gesture = GestureClick::new();
    gesture.set_button(3); // Right click
    let window_clone = window.clone();
    let sender_clone = sender.clone();
    
    gesture.connect_pressed(move |_gesture, _, _, _| {
        // Simple menu simulation for now since PopoverMenu requires a MenuModel
        // We'll just toggle the settings window on right click for demonstration
        // In a real app, we'd build a proper Gio::Menu
        crate::settings_ui::show_settings_window(&window_clone);
        crate::shortcuts_ui::show_shortcuts_panel(&window_clone, sender_clone.clone());
    });
    window.add_controller(gesture);

    window.present();

    // --- Update Loop ---
    let monitor = Rc::new(RefCell::new(SystemMonitor::new()));
    
    glib::timeout_add_seconds_local(1, move || {
        let mut mon = monitor.borrow_mut();
        mon.refresh();
        
        // CPU
        let cpu = mon.get_cpu_usage();
        cpu_value.set_text(&format!("{:.0}%", cpu));
        // Color logic
        if cpu > 80.0 { cpu_value.add_css_class("status-critical"); }
        else if cpu > 50.0 { cpu_value.add_css_class("status-warning"); }
        else { cpu_value.remove_css_class("status-critical"); cpu_value.remove_css_class("status-warning"); }

        // GPU
        if let Some(gpu) = mon.get_gpu_usage() {
            gpu_value.set_text(&format!("{:.0}%", gpu));
        }

        // RAM
        let (used, _total) = mon.get_ram_usage();
        let used_gb = used as f64 / 1024.0 / 1024.0 / 1024.0;
        ram_value.set_text(&format!("{:.1} GB", used_gb));

        // Net
        let (rx, tx) = mon.get_network_stats();
        let total_speed = (rx + tx) as f64 / 1024.0; // KB/s
        net_value.set_text(&format!("{:.0} KB/s", total_speed));

        glib::ControlFlow::Continue
    });
}
