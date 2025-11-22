use gtk4::prelude::*;
use gtk4::{
    Align, Box, Button, ComboBoxText, Label, Notebook, Orientation, Switch, TextView, Window,
};
use std::{cell::RefCell, rc::Rc};

use crate::settings::{MonitorStyle, Settings, Theme};
use font_kit::source::SystemSource;

#[allow(clippy::too_many_lines)]
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
    let theme_group = Box::new(Orientation::Vertical, 6);
    theme_group.append(&Label::new(Some("Theme")));
    
    let theme_box = Box::new(Orientation::Horizontal, 10);
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
    
    // Preview swatch (placeholder for now)
    let swatch = gtk4::DrawingArea::new();
    swatch.set_content_width(24);
    swatch.set_content_height(24);
    swatch.add_css_class("theme-swatch");
    theme_box.append(&swatch);
    
    theme_group.append(&theme_box);
    main_box.append(&theme_group);

    // Monitor style
    let style_group = Box::new(Orientation::Vertical, 6);
    style_group.append(&Label::new(Some("Monitoring Style")));
    
    let style_box = Box::new(Orientation::Horizontal, 10);
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
    
    // Sparkline demo
    let demo_chart = gtk4::DrawingArea::new();
    demo_chart.set_content_width(60);
    demo_chart.set_content_height(24);
    demo_chart.add_css_class("demo-chart");
    style_box.append(&demo_chart);
    
    style_group.append(&style_box);
    main_box.append(&style_group);

    // Sections
    let visibility_expander = gtk4::Expander::new(Some("Visibility"));
    let visibility_box = Box::new(Orientation::Vertical, 6);
    
    let terminal_toggle = create_toggle("Show terminal", settings_snapshot.show_terminal);
    let monitoring_toggle = create_toggle("Show monitoring", settings_snapshot.show_monitoring);
    let shortcuts_toggle = create_toggle(
        "Show shortcuts panel",
        settings_snapshot.show_shortcuts_panel,
    );
    
    visibility_box.append(&terminal_toggle.0);
    visibility_box.append(&monitoring_toggle.0);
    visibility_box.append(&shortcuts_toggle.0);
    visibility_expander.set_child(Some(&visibility_box));
    visibility_expander.set_expanded(true);
    main_box.append(&visibility_expander);

    let sensors_expander = gtk4::Expander::new(Some("Sensors"));
    let sensors_box = Box::new(Orientation::Vertical, 6);
    let cpu_box = create_toggle("Show CPU", settings_snapshot.show_cpu);
    let gpu_box = create_toggle("Show GPU", settings_snapshot.show_gpu);
    let ram_box = create_toggle("Show RAM", settings_snapshot.show_ram);
    let net_box = create_toggle("Show Network", settings_snapshot.show_network);
    
    sensors_box.append(&cpu_box.0);
    sensors_box.append(&gpu_box.0);
    sensors_box.append(&ram_box.0);
    sensors_box.append(&net_box.0);
    sensors_expander.set_child(Some(&sensors_box));
    main_box.append(&sensors_expander);

    let system_expander = gtk4::Expander::new(Some("System"));
    let system_box = Box::new(Orientation::Vertical, 6);
    let auto_start_box = create_toggle("Launch at start", settings_snapshot.launch_at_start);
    let lock_place_box = create_toggle("Lock position", settings_snapshot.lock_in_place);
    let lock_size_box = create_toggle("Lock size", settings_snapshot.lock_size);
    
    let shell_box = Box::new(Orientation::Horizontal, 10);
    shell_box.append(&Label::new(Some("Shell")));
    let shell_entry = gtk4::Entry::new();
    shell_entry.set_text(&settings_snapshot.shell);
    shell_entry.set_hexpand(true);
    shell_box.append(&shell_entry);

    system_box.append(&auto_start_box.0);
    system_box.append(&lock_place_box.0);
    system_box.append(&lock_size_box.0);
    system_box.append(&shell_box);
    system_expander.set_child(Some(&system_box));
    main_box.append(&system_expander);

    // Styling Tab (Created early to be captured by save closure)
    let styling_box = Box::new(Orientation::Vertical, 12);
    styling_box.set_margin_top(16);
    styling_box.set_margin_bottom(16);
    styling_box.set_margin_start(16);
    styling_box.set_margin_end(16);
    
    let styling_scroll = gtk4::ScrolledWindow::new();
    styling_scroll.set_child(Some(&styling_box));
    styling_scroll.set_vexpand(true);

fn get_system_fonts() -> Vec<String> {
    let source = SystemSource::new();
    let mut families = source.all_families().unwrap_or_else(|_| vec!["Sans".to_string(), "Monospace".to_string()]);
    families.sort();
    families.dedup();
    families
}

    // Helper to create section controls
    let fonts = get_system_fonts();
    let create_section_controls = |title: &str, style: &crate::settings::SectionStyle| {
        let group = Box::new(Orientation::Vertical, 6);
        group.append(&Label::new(Some(title)));
        
        let grid = gtk4::Grid::new();
        grid.set_column_spacing(10);
        grid.set_row_spacing(6);
        
        // Opacity
        grid.attach(&Label::new(Some("Opacity")), 0, 0, 1, 1);
        let opacity = gtk4::Scale::with_range(Orientation::Horizontal, 0.1, 1.0, 0.05);
        opacity.set_value(style.opacity);
        opacity.set_hexpand(true);
        grid.attach(&opacity, 1, 0, 1, 1);
        
        // Color
        grid.attach(&Label::new(Some("Bg Color")), 0, 1, 1, 1);
        let color = gtk4::Entry::new();
        color.set_text(&style.bg_color);
        grid.attach(&color, 1, 1, 1, 1);
        
        // Font Size
        grid.attach(&Label::new(Some("Font Size")), 0, 2, 1, 1);
        let font_size = gtk4::Scale::with_range(Orientation::Horizontal, 8.0, 24.0, 1.0);
        font_size.set_value(style.font_size);
        font_size.set_hexpand(true);
        grid.attach(&font_size, 1, 2, 1, 1);

        // Font Family
        grid.attach(&Label::new(Some("Font Family")), 0, 3, 1, 1);
        let font_combo = ComboBoxText::new();
        let current_font = &style.font_family;
        let mut active_id = 0;
        
        for (i, family) in fonts.iter().enumerate() {
            font_combo.append_text(family);
            if family == current_font {
                active_id = u32::try_from(i).unwrap_or(0);
            }
        }
        // If current font not found, add it (custom or fallback)
        if font_combo.active_text().as_deref() != Some(current_font) {
             // Try to find it again by text, if not found, maybe it's not in the list?
             // For simplicity, just select the one we found or 0
             font_combo.set_active(Some(active_id));
        }
        
        // Better way: set active by ID if we used IDs, but here we just rely on index matching sorted list
        // Re-check active
        if let Some(idx) = fonts.iter().position(|f| f == current_font) {
            font_combo.set_active(Some(u32::try_from(idx).unwrap_or(0)));
        } else {
             // Add as custom option? Or just select first.
             font_combo.set_active(Some(0));
        }
        font_combo.set_hexpand(true);
        grid.attach(&font_combo, 1, 3, 1, 1);

        // Radius
        grid.attach(&Label::new(Some("Radius")), 0, 4, 1, 1);
        let radius = gtk4::Scale::with_range(Orientation::Horizontal, 0.0, 50.0, 1.0);
        radius.set_value(style.border_radius);
        radius.set_hexpand(true);
        grid.attach(&radius, 1, 4, 1, 1);
        
        group.append(&grid);
        group.append(&gtk4::Separator::new(Orientation::Horizontal));
        (group, opacity, color, font_size, font_combo, radius)
    };

    let (term_box, term_op, term_col, term_size, term_font, term_rad) = 
        create_section_controls("Terminal", &settings_snapshot.terminal_style);
    styling_box.append(&term_box);

    let (mon_box, mon_op, mon_col, mon_size, mon_font, mon_rad) = 
        create_section_controls("Monitoring", &settings_snapshot.monitoring_style);
    styling_box.append(&mon_box);

    let (sc_box, sc_op, sc_col, sc_size, sc_font, sc_rad) = 
        create_section_controls("Shortcuts", &settings_snapshot.shortcuts_style);
    styling_box.append(&sc_box);

    // Save Button
    // TODO(senior-ui): Swap to Apply/Reset buttons with undo to encourage experimentation.
    // Save / Reset Buttons
    let action_box = Box::new(Orientation::Horizontal, 10);
    action_box.set_halign(Align::End);
    
    let reset_btn = Button::with_label("Reset");
    let save_btn = Button::with_label("Apply");
    save_btn.add_css_class("pill-btn");
    
    action_box.append(&reset_btn);
    action_box.append(&save_btn);
    main_box.append(&action_box);
    
    let window_clone = window.clone();

    {
        let window_clone = window.clone();
        reset_btn.connect_clicked(move |_| {
            // Simply close without saving to revert
            window_clone.close();
        });
    }

    save_btn.connect_clicked(move |_| {
        let mut new_settings = settings_snapshot.clone();

        // Theme
        new_settings.theme = match theme_combo.active().unwrap_or(0) {
            #[allow(clippy::match_same_arms)]
            0 | 3 => Theme::Dark, // Default to Dark for 0 and fallback
            1 => Theme::Light,
            2 => Theme::Solarized,
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
        new_settings.shell = shell_entry.text().to_string();

        // Styling
        new_settings.terminal_style.opacity = term_op.value();
        new_settings.terminal_style.bg_color = term_col.text().to_string();
        new_settings.terminal_style.font_size = term_size.value();
        new_settings.terminal_style.font_family = term_font.active_text().unwrap_or_else(|| "Monospace".into()).to_string();
        new_settings.terminal_style.border_radius = term_rad.value();

        new_settings.monitoring_style.opacity = mon_op.value();
        new_settings.monitoring_style.bg_color = mon_col.text().to_string();
        new_settings.monitoring_style.font_size = mon_size.value();
        new_settings.monitoring_style.font_family = mon_font.active_text().unwrap_or_else(|| "Sans".into()).to_string();
        new_settings.monitoring_style.border_radius = mon_rad.value();

        new_settings.shortcuts_style.opacity = sc_op.value();
        new_settings.shortcuts_style.bg_color = sc_col.text().to_string();
        new_settings.shortcuts_style.font_size = sc_size.value();
        new_settings.shortcuts_style.font_family = sc_font.active_text().unwrap_or_else(|| "Sans".into()).to_string();
        new_settings.shortcuts_style.border_radius = sc_rad.value();

        settings.replace(new_settings.clone());
        new_settings.save();
        on_save(new_settings);
        window_clone.close();
    });

    notebook.append_page(&main_box, Some(&Label::new(Some("Controls"))));

    notebook.append_page(&styling_scroll, Some(&Label::new(Some("Styling"))));

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
         - Toggle themes here or via CSS in src/style.css.\n\
         \n\
         Documentation available at: /usr/share/doc/vitray-widget/",
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
