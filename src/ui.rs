use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box, Label, Orientation, ProgressBar, CssProvider};
use crate::monitor::SystemMonitor;
use crate::terminal::create_terminal;
use std::rc::Rc;
use std::cell::RefCell;
use gtk4::glib;

pub fn build_ui(app: &Application) {
    let provider = CssProvider::new();
    let provider = CssProvider::new();
    // Try local path first (dev), then system path (installed)
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
    window.set_default_size(400, 600);
    
    window.set_decorated(false);
    window.set_css_classes(&["glass-window"]);

    let main_box = Box::new(Orientation::Vertical, 10);
    main_box.add_css_class("glass-panel");
    
    let cpu_label = Label::new(Some("CPU: 0%"));
    cpu_label.add_css_class("monitor-label");
    let cpu_bar = ProgressBar::new();
    
    let ram_label = Label::new(Some("RAM: 0 / 0 GB"));
    ram_label.add_css_class("monitor-label");
    let ram_bar = ProgressBar::new();

    let net_label = Label::new(Some("Net: ↓ 0 KB/s  ↑ 0 KB/s"));
    net_label.add_css_class("monitor-label");

    main_box.append(&cpu_label);
    main_box.append(&cpu_bar);
    main_box.append(&ram_label);
    main_box.append(&ram_bar);
    main_box.append(&net_label);

    let terminal = create_terminal();
    let term_box = Box::new(Orientation::Vertical, 0);
    term_box.add_css_class("terminal-container");
    
    let scrolled = gtk4::ScrolledWindow::new();
    scrolled.set_child(Some(&terminal));
    scrolled.set_vexpand(true);
    
    term_box.append(&scrolled);
    main_box.append(&term_box);

    window.set_child(Some(&main_box));
    window.present();

    let monitor = Rc::new(RefCell::new(SystemMonitor::new()));
    
    glib::timeout_add_seconds_local(1, move || {
        let mut mon = monitor.borrow_mut();
        mon.refresh();
        
        let cpu = mon.get_cpu_usage();
        cpu_label.set_text(&format!("CPU: {:.1}%", cpu));
        cpu_bar.set_fraction(cpu as f64 / 100.0);
        
        let (used, total) = mon.get_ram_usage();
        let used_gb = used as f64 / 1024.0 / 1024.0 / 1024.0;
        let total_gb = total as f64 / 1024.0 / 1024.0 / 1024.0;
        ram_label.set_text(&format!("RAM: {:.1} / {:.1} GB", used_gb, total_gb));
        ram_bar.set_fraction(used as f64 / total as f64);
        
        let (rx, tx) = mon.get_network_stats();
        let rx_kb = rx as f64 / 1024.0;
        let tx_kb = tx as f64 / 1024.0;
        net_label.set_text(&format!("Net: ↓ {:.1} KB/s  ↑ {:.1} KB/s", rx_kb, tx_kb));
        
        glib::ControlFlow::Continue
    });
}
