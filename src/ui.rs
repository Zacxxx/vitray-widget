use gtk4::gdk::prelude::*;
use gtk4::prelude::*;
use gtk4::{
    gdk, glib, Align, Application, ApplicationWindow, Box, Button, CssProvider, DrawingArea,
    GestureClick, Grid, Label, LevelBar, Notebook, Orientation, Popover, Stack, StackTransitionType,
};
use std::{cell::RefCell, rc::Rc};
use vte4::TerminalExt;

use crate::settings::{MonitorStyle, Settings, Theme};
use crate::settings_ui::show_settings_window;
use crate::shortcuts::Shortcuts;
use crate::shortcuts_ui::ShortcutsPanel;
use crate::terminal::create_terminal;

#[derive(Clone, Copy)]
enum Trend {
    Up,
    Down,
    Stable,
}

#[derive(Clone)]
struct PerformanceStrip {
    cpu: Button,
    gpu: Button,
    ram: Button,
    net: Button,
}

impl PerformanceStrip {
    fn new() -> Self {
        let chip = |text: &str, tooltip: &str| {
            let btn = Button::with_label(text);
            btn.add_css_class("perf-chip");
            btn.add_css_class("flat");
            btn.set_tooltip_text(Some(tooltip));
            btn
        };

        Self {
            cpu: chip("CPU —", "CPU Usage\nClick for details"),
            gpu: chip("GPU —", "GPU Usage\nClick for details"),
            ram: chip("RAM —", "Memory Usage\nClick for details"),
            net: chip("NET —", "Network Traffic\nClick for details"),
        }
    }

    fn widget(&self) -> Box {
        let row = Box::new(Orientation::Horizontal, 6);
        row.set_halign(Align::Center);
        row.add_css_class("perf-ribbon");
        row.append(&self.cpu);
        row.append(&self.gpu);
        row.append(&self.ram);
        row.append(&self.net);
        row
    }

    fn update(&self, cpu: &str, gpu: &str, ram: &str, net: &str) {
        self.cpu.set_label(cpu);
        self.gpu.set_label(gpu);
        self.ram.set_label(ram);
        self.net.set_label(net);
    }
}

#[derive(Clone)]
struct MonitorCard {
    container: Box,
    value_label: Label,
    bar: LevelBar,
    chart: DrawingArea,
    stack: Stack,
    history: Rc<RefCell<Vec<f64>>>,
    last_value: Rc<RefCell<f64>>,
    scale_max: f64,
}

impl MonitorCard {
    fn new(title: &str, style: &MonitorStyle, scale_max: f64) -> Self {
        let container = Box::new(Orientation::Vertical, 4);
        container.add_css_class("monitor-card");

        let title_label = Label::new(Some(title));
        title_label.add_css_class("card-title");
        title_label.set_halign(Align::Start);
        container.append(&title_label);

        let value_label = Label::new(Some("—"));
        value_label.add_css_class("card-value");
        value_label.set_halign(Align::Start);
        container.append(&value_label);

        let stack = Stack::new();
        stack.set_transition_type(StackTransitionType::Crossfade);
        stack.set_vexpand(true);
        stack.set_hexpand(true);

        let spacer = Box::new(Orientation::Vertical, 0);
        stack.add_named(&spacer, Some("text"));

        let bar = LevelBar::new();
        bar.set_min_value(0.0);
        bar.set_max_value(scale_max);
        bar.add_css_class("monitor-bar");
        stack.add_named(&bar, Some("bar"));

        let chart = DrawingArea::new();
        chart.add_css_class("monitor-chart");
        // TODO(senior-ui): Swap to GtkPlot or custom GPU-accelerated canvas so historical data looks
        // smooth and can display tooltips on hover.
        stack.add_named(&chart, Some("chart"));

        container.append(&stack);

        let history = Rc::new(RefCell::new(Vec::<f64>::new()));
        let last_value = Rc::new(RefCell::new(0.0));

        let card = Self {
            container,
            value_label,
            bar,
            chart,
            stack,
            history,
            last_value,
            scale_max,
        };
        card.set_style(style);
        card.install_chart_drawer();
        card
    }

    fn install_chart_drawer(&self) {
        let hist = self.history.clone();
        self.chart.set_draw_func(move |_area, cr, width, height| {
            let data = hist.borrow();
            if data.len() < 2 {
                return;
            }

            let max = data
                .iter()
                .cloned()
                .fold(1.0_f64, |a, b| if b > a { b } else { a });
            let step = width as f64 / (data.len() - 1) as f64;

            // Fill path
            cr.set_source_rgba(0.2, 0.6, 1.0, 0.15);
            cr.move_to(0.0, height as f64);
            
            for (idx, val) in data.iter().enumerate() {
                let x = idx as f64 * step;
                let y = height as f64 - ((val / max) * height as f64 * 0.95);
                cr.line_to(x, y);
            }
            cr.line_to(width as f64, height as f64);
            cr.close_path();
            let _ = cr.fill();

            // Stroke path
            cr.set_source_rgba(0.2, 0.6, 1.0, 0.8);
            cr.set_line_width(2.0);
            
            for (idx, val) in data.iter().enumerate() {
                let x = idx as f64 * step;
                let y = height as f64 - ((val / max) * height as f64 * 0.95);
                if idx == 0 {
                    cr.move_to(x, y);
                } else {
                    cr.line_to(x, y);
                }
            }
            let _ = cr.stroke();
        });
    }

    fn set_style(&self, style: &MonitorStyle) {
        match style {
            MonitorStyle::Bar => self.stack.set_visible_child_name("bar"),
            MonitorStyle::Chart => self.stack.set_visible_child_name("chart"),
            MonitorStyle::Text => self.stack.set_visible_child_name("text"),
        }
    }

    fn update(&self, numeric: f64, display: &str, style: &MonitorStyle) {
        self.set_style(style);
        self.value_label.set_text(display);

        let mut last = self.last_value.borrow_mut();
        let trend = if *last == 0.0 {
            Trend::Stable
        } else if numeric > *last + 1.0 {
            Trend::Up
        } else if numeric + 1.0 < *last {
            Trend::Down
        } else {
            Trend::Stable
        };
        *last = numeric;

        self.value_label.remove_css_class("trend-up");
        self.value_label.remove_css_class("trend-down");
        self.value_label.remove_css_class("trend-stable");
        self.value_label.add_css_class(match trend {
            Trend::Up => "trend-up",
            Trend::Down => "trend-down",
            Trend::Stable => "trend-stable",
        });

        let clamped = numeric.min(self.scale_max);
        self.bar.set_value(clamped);

        {
            let mut hist = self.history.borrow_mut();
            hist.push(clamped);
            
            // Adaptive history based on width (approx 1px per sample)
            let width = self.chart.width();
            let max_samples = if width > 0 { width as usize } else { 120 };
            
            if hist.len() > max_samples {
                let remove_count = hist.len() - max_samples;
                for _ in 0..remove_count {
                     hist.remove(0);
                }
            }
        }
        self.chart.queue_draw();
    }

    fn set_visible(&self, show: bool) {
        self.container.set_visible(show);
    }
}

#[derive(Clone)]
struct MonitorGroup {
    cpu: MonitorCard,
    gpu: MonitorCard,
    ram: MonitorCard,
    net: MonitorCard,
}

impl MonitorGroup {
    fn set_style(&self, style: &MonitorStyle) {
        self.cpu.set_style(style);
        self.gpu.set_style(style);
        self.ram.set_style(style);
        self.net.set_style(style);
    }

    fn set_visibility(&self, settings: &Settings) {
        self.cpu.set_visible(settings.show_cpu);
        self.gpu.set_visible(settings.show_gpu);
        self.ram.set_visible(settings.show_ram);
        self.net.set_visible(settings.show_network);
    }
}

#[derive(Clone)]
struct UiHandles {
    main_window: ApplicationWindow,
    terminal_window: ApplicationWindow,
    monitor_window: ApplicationWindow,
    shortcuts_window: ApplicationWindow,
    monitor_cards: MonitorGroup,
    performance_strip: PerformanceStrip,
    shortcuts_panel: ShortcutsPanel,
    settings: Rc<RefCell<Settings>>,
    style_provider: CssProvider,
}

#[derive(Clone)]
struct HeaderBar {
    widget: Box,
    settings_btn: Button,
    shortcuts_btn: Button,
}

pub fn build_ui(app: &Application) {
    let settings = Rc::new(RefCell::new(Settings::load()));
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

    // Dynamic style provider for user settings
    let dynamic_provider = CssProvider::new();
    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &dynamic_provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION + 1,
        );
    }

    // --- Main Control Window ---
    let main_window = ApplicationWindow::new(app);
    main_window.set_title(Some("Vitray Control"));
    main_window.set_default_size(600, 100);
    main_window.set_decorated(false); // Keep it clean
    main_window.add_css_class("glass-window");
    
    // --- Terminal Window ---
    let terminal_window = create_standalone_window(app, "Terminal", 600, 400);
    terminal_window.add_css_class("terminal-window");

    // --- Monitor Window ---
    let monitor_window = create_standalone_window(app, "Vitals", 600, 300);
    monitor_window.add_css_class("monitor-window");

    // --- Shortcuts Window ---
    let shortcuts_window = create_standalone_window(app, "Shortcuts", 300, 500);
    shortcuts_window.add_css_class("shortcuts-window");


    // --- Terminal channel ---
    let (sender, receiver) = async_channel::unbounded::<String>();


    // --- Header & Perf Strip (Main Window Content) ---
    let main_content = Box::new(Orientation::Vertical, 0);
    main_content.add_css_class("glass-panel");
    
    let performance_strip = PerformanceStrip::new();
    let header = build_header(&main_window, settings.clone());
    main_content.append(&header.widget);
    main_content.append(&performance_strip.widget());
    main_window.set_child(Some(&main_content));

    // --- Terminal Section Content ---
    let terminal_section = Box::new(Orientation::Vertical, 6);
    terminal_section.add_css_class("terminal-section");
    
    let terminal_header = Box::new(Orientation::Horizontal, 8);
    terminal_header.add_css_class("terminal-header");
    let tabs_btn = Button::with_label("+ Tab");
    tabs_btn.add_css_class("pill-btn");
    
    let notebook = Notebook::new();
    notebook.set_show_tabs(true);
    notebook.set_scrollable(true);
    notebook.add_css_class("terminal-notebook");
    notebook.set_group_name(Some("terminal-tabs"));
    let shortcuts_btn = Button::with_label("Shortcuts");
    shortcuts_btn.add_css_class("pill-btn");
    terminal_header.append(&Label::new(Some("Terminal")));
    terminal_header.append(&tabs_btn);
    terminal_header.append(&shortcuts_btn);
    terminal_header.set_halign(Align::Start);

    // Initial tab
    let shell = settings.borrow().shell.clone();
    let terminal = create_terminal(&shell, None, None);
    let scrolled = gtk4::ScrolledWindow::new();
    scrolled.set_child(Some(&terminal));
    scrolled.set_vexpand(true);
    
    let tab_label = build_tab_label(&notebook, &scrolled, "Terminal");
    notebook.append_page(&scrolled, Some(&tab_label));
    notebook.set_tab_reorderable(&scrolled, true);
    notebook.set_tab_detachable(&scrolled, true);

    terminal_section.append(&terminal_header);
    terminal_section.append(&notebook);
    terminal_window.set_child(Some(&terminal_section));

    // --- Monitoring Section Content ---
    let monitoring_section = Box::new(Orientation::Vertical, 8);
    monitoring_section.add_css_class("monitoring-shell");
    monitoring_section.add_css_class("glass-panel");
    monitoring_section.append(&Label::new(Some("Vitals")));

    let grid = Grid::new();
    grid.set_column_spacing(12);
    grid.set_row_spacing(12);
    grid.set_margin_bottom(6);
    grid.set_margin_start(4);
    grid.set_margin_end(4);
    grid.add_css_class("monitoring-grid");

    let monitor_cards = MonitorGroup {
        cpu: MonitorCard::new("CPU", &settings.borrow().monitor_style, 100.0),
        gpu: MonitorCard::new("GPU", &settings.borrow().monitor_style, 100.0),
        ram: MonitorCard::new("RAM", &settings.borrow().monitor_style, 100.0),
        net: MonitorCard::new("Network", &settings.borrow().monitor_style, 2000.0),
    };

    grid.attach(&monitor_cards.cpu.container, 0, 0, 1, 1);
    grid.attach(&monitor_cards.gpu.container, 1, 0, 1, 1);
    grid.attach(&monitor_cards.ram.container, 0, 1, 1, 1);
    grid.attach(&monitor_cards.net.container, 1, 1, 1, 1);
    monitoring_section.append(&grid);
    monitor_window.set_child(Some(&monitoring_section));

    // --- Shortcuts Content ---
    let shortcuts_panel = ShortcutsPanel::new(&main_window, sender.clone());
    shortcuts_panel.set_revealed(true); // Always visible in its own window
    
    let shortcuts_wrapper = Box::new(Orientation::Vertical, 0);
    shortcuts_wrapper.add_css_class("glass-panel");
    shortcuts_wrapper.add_css_class("shortcuts-section");
    shortcuts_wrapper.append(&shortcuts_panel.revealer);
    shortcuts_window.set_child(Some(&shortcuts_wrapper));


    // Terminal channel feed
    glib::MainContext::default().spawn_local(async move {
        while let Ok(cmd) = receiver.recv().await {
            let cmd_with_newline = format!("{}\n", cmd);
            terminal.feed_child(cmd_with_newline.as_bytes());
        }
    });

    {
        let notebook_clone = notebook.clone();
        let settings_clone = settings.clone();
        tabs_btn.connect_clicked(move |_| {
            let shell = settings_clone.borrow().shell.clone();
            add_terminal_tab(&notebook_clone, &shell);
        });
    }

    {
        let win = shortcuts_window.clone();
        shortcuts_btn.connect_clicked(move |_| {
            if win.is_visible() {
                win.hide();
            } else {
                win.present();
            }
        });
    }

    let handles = UiHandles {
        main_window: main_window.clone(),
        terminal_window: terminal_window.clone(),
        monitor_window: monitor_window.clone(),
        shortcuts_window: shortcuts_window.clone(),
        monitor_cards: monitor_cards.clone(),
        performance_strip: performance_strip.clone(),
        shortcuts_panel: shortcuts_panel.clone(),
        settings: settings.clone(),
        style_provider: dynamic_provider,
    };

    // Context menu / right click (Attached to main window for now)
    build_context_menu(handles.clone());

    {
        let handles_clone = handles.clone();
        header.settings_btn.connect_clicked(move |_| {
            let settings_rc = handles_clone.settings.clone();
            let handles_apply = handles_clone.clone();
            show_settings_window(
                &handles_clone.main_window,
                settings_rc,
                move |updated: Settings| {
                    handles_apply.settings.replace(updated.clone());
                    apply_settings(&handles_apply, &updated);
                },
            );
        });
    }

    {
        let win = handles.shortcuts_window.clone();
        header
            .shortcuts_btn
            .connect_clicked(move |_| {
                 if win.is_visible() {
                    win.hide();
                } else {
                    win.present();
                }
            });
    }
    
    apply_settings(&handles, &settings.borrow());

    main_window.present();


    // --- Update Loop (Async) ---
    let monitor_receiver = crate::monitor::start_monitoring_service();
    let mut last_net: Option<(u64, u64)> = None;

    let handles_weak = handles.clone(); // Actually we need strong reference or weak? 
    // spawn_local keeps the future alive. We need to move handles in.
    // But handles is Clone.
    
    glib::MainContext::default().spawn_local(async move {
        while let Ok(data) = monitor_receiver.recv().await {
            let active_settings = handles_weak.settings.borrow().clone();
            let style = active_settings.monitor_style.clone();

            let cpu = data.cpu_usage as f64;
            handles_weak
                .monitor_cards
                .cpu
                .update(cpu, &format!("{cpu:.0}%"), &style);

            let gpu_usage = data.gpu_usage;
            if let Some(gpu) = gpu_usage {
                handles_weak
                    .monitor_cards
                    .gpu
                    .update(gpu as f64, &format!("{gpu:.0}%"), &style);
            } else {
                handles_weak.monitor_cards.gpu.update(0.0, "N/A", &style);
            }

            let (used, total) = (data.ram_used, data.ram_total);
            let used_gb = used as f64 / 1024.0 / 1024.0 / 1024.0;
            let total_gb = total as f64 / 1024.0 / 1024.0 / 1024.0;
            let ram_pct = if total > 0 {
                used as f64 / total as f64 * 100.0
            } else {
                0.0
            };
            handles_weak.monitor_cards.ram.update(
                ram_pct,
                &format!("{used_gb:.1}/{total_gb:.1} GB"),
                &style,
            );

            let (rx_raw, tx_raw) = (data.rx_bytes, data.tx_bytes);
            let (rx_rate, tx_rate) = if let Some((prev_rx, prev_tx)) = last_net {
                (
                    rx_raw.saturating_sub(prev_rx),
                    tx_raw.saturating_sub(prev_tx),
                )
            } else {
                (0, 0)
            };
            last_net = Some((rx_raw, tx_raw));
            let total_speed = (rx_rate + tx_rate) as f64 / 1024.0;
            let rx_kb = rx_rate as f64 / 1024.0;
            let tx_kb = tx_rate as f64 / 1024.0;
            
            let rx_display = if rx_kb > 1024.0 {
                format!("{:.1} MB/s", rx_kb / 1024.0)
            } else {
                format!("{:.0} KB/s", rx_kb)
            };
            
            let tx_display = if tx_kb > 1024.0 {
                format!("{:.1} MB/s", tx_kb / 1024.0)
            } else {
                format!("{:.0} KB/s", tx_kb)
            };

            handles_weak.monitor_cards.net.update(
                total_speed.min(2000.0),
                &format!("↓{} ↑{}", rx_display, tx_display),
                &style,
            );

            let gpu_label = gpu_usage.map(|gpu| format!("GPU {:.0}%", gpu));
            let net_label = format!("NET {total_speed:.0} KB/s");

            handles_weak.performance_strip.update(
                &format!("CPU {cpu:.0}%"),
                gpu_label.as_deref().unwrap_or("GPU —"),
                &format!("RAM {ram_pct:.0}%"),
                &net_label,
            );
        }
    });
}

fn add_terminal_tab(notebook: &Notebook, shell: &str) {
    let terminal = create_terminal(shell, None, None);
    let scrolled = gtk4::ScrolledWindow::new();
    scrolled.set_child(Some(&terminal));
    scrolled.set_vexpand(true);
    
    let idx = notebook.n_pages() + 1;
    let title = format!("T{}", idx);
    let tab_label = build_tab_label(notebook, &scrolled, &title);
    
    notebook.append_page(&scrolled, Some(&tab_label));
    notebook.set_tab_reorderable(&scrolled, true);
    notebook.set_tab_detachable(&scrolled, true);
    notebook.set_current_page(Some(notebook.n_pages() - 1));
}

fn build_tab_label(notebook: &Notebook, page: &impl IsA<gtk4::Widget>, title: &str) -> Box {
    let box_ = Box::new(Orientation::Horizontal, 4);
    let label = Label::new(Some(title));
    box_.append(&label);
    
    let close_btn = Button::from_icon_name("window-close-symbolic");
    close_btn.add_css_class("flat");
    close_btn.add_css_class("small-icon");
    box_.append(&close_btn);
    
    let notebook_clone = notebook.clone();
    let page_clone = page.clone();
    close_btn.connect_clicked(move |_| {
        if let Some(idx) = notebook_clone.page_num(&page_clone) {
            notebook_clone.remove_page(Some(idx));
        }
    });
    
    // Rename on double click
    let gesture = GestureClick::new();
    gesture.set_button(1);
    let label_clone = label.clone();
    let parent_win = notebook.root().and_then(|r| r.downcast::<gtk4::Window>().ok());
    
    gesture.connect_pressed(move |_gesture, n_press, _, _| {
        if n_press == 2 {
            // Simple rename dialog
            if let Some(win) = &parent_win {
                prompt_rename(win, &label_clone);
            }
        }
    });
    box_.add_controller(gesture);
    
    box_
}

fn prompt_rename(parent: &gtk4::Window, label: &Label) {
    let dialog = gtk4::Dialog::builder()
        .transient_for(parent)
        .modal(true)
        .title("Rename Tab")
        .build();
        
    let entry = gtk4::Entry::new();
    entry.set_text(&label.text());
    entry.set_activates_default(true);
    
    let content = dialog.content_area();
    content.set_margin_top(10);
    content.set_margin_bottom(10);
    content.set_margin_start(10);
    content.set_margin_end(10);
    content.set_spacing(10);
    content.append(&entry);
    
    let btn = dialog.add_button("Rename", gtk4::ResponseType::Ok);
    btn.add_css_class("suggested-action");
    dialog.set_default_response(gtk4::ResponseType::Ok);
    
    let label_clone = label.clone();
    dialog.connect_response(move |d, resp| {
        if resp == gtk4::ResponseType::Ok {
            let text = entry.text();
            if !text.is_empty() {
                label_clone.set_text(&text);
            }
        }
        d.close();
    });
    
    dialog.show();
}

fn apply_settings(handles: &UiHandles, settings: &Settings) {
    // Main window theme (and others if we want)
    apply_theme_fixed(&handles.main_window, &settings.theme);
    apply_theme_fixed(&handles.terminal_window, &settings.theme);
    apply_theme_fixed(&handles.monitor_window, &settings.theme);
    apply_theme_fixed(&handles.shortcuts_window, &settings.theme);

    handles.terminal_window.set_visible(settings.show_terminal);
    handles.monitor_window.set_visible(settings.show_monitoring);
    handles.shortcuts_window.set_visible(settings.show_shortcuts_panel);
    
    handles.monitor_cards.set_visibility(settings);
    
    // Lock size logic might need to change for multiple windows, 
    // or we just apply it to all.
    handles.main_window.set_resizable(!settings.lock_size);
    handles.terminal_window.set_resizable(!settings.lock_size);
    handles.monitor_window.set_resizable(!settings.lock_size);
    handles.shortcuts_window.set_resizable(!settings.lock_size);

    handles.monitor_cards.set_style(&settings.monitor_style);
    apply_dynamic_styles(&handles.style_provider, settings);
}

fn apply_dynamic_styles(provider: &CssProvider, settings: &Settings) {
    let gen_css = |selector: &str, style: &crate::settings::SectionStyle| {
        format!(
            "{} {{ background-color: {}; opacity: {}; font-size: {}px; font-family: '{}'; border-radius: {}px; }}",
            selector, style.bg_color, style.opacity, style.font_size, style.font_family, style.border_radius
        )
    };

    // Note: We target the specific classes we added.
    // For opacity, we might need to be careful as it affects children. 
    // If the user wants background opacity only, we should use rgba in bg_color.
    // But the requirement says "control opacity", which usually implies the whole widget or background.
    // Given the glassmorphism, background opacity is usually handled via the color alpha.
    // If 'opacity' here means the widget opacity, it will fade content too.
    // Let's assume widget opacity for now as it's requested as a separate control.
    
    let css = format!(
        "{}\n{}\n{}",
        gen_css(".terminal-section", &settings.terminal_style),
        gen_css(".monitoring-shell", &settings.monitoring_style),
        gen_css(".shortcuts-section", &settings.shortcuts_style)
    );

    provider.load_from_data(&css);
}



fn apply_theme_fixed(window: &ApplicationWindow, theme: &Theme) {
    for cls in &[
        "theme-dark",
        "theme-light",
        "theme-solarized",
        "theme-tokyo",
    ] {
        window.remove_css_class(cls);
    }
    let class_name = match theme {
        Theme::Dark => "theme-dark",
        Theme::Light => "theme-light",
        Theme::Solarized => "theme-solarized",
        Theme::Tokyo => "theme-tokyo",
    };
    window.add_css_class(class_name);
}

fn build_header(window: &ApplicationWindow, settings: Rc<RefCell<Settings>>) -> HeaderBar {
    let header = Box::new(Orientation::Horizontal, 8);
    header.add_css_class("window-header");

    let title = Label::new(Some("Vitray"));
    title.add_css_class("title");
    
    let time_label = Label::new(None);
    time_label.add_css_class("header-time");
    
    // Update time every second
    glib::timeout_add_seconds_local(1, {
        let label = time_label.clone();
        move || {
            let now = glib::DateTime::now_local().unwrap();
            label.set_text(&now.format("%H:%M").unwrap().to_string());
            glib::ControlFlow::Continue
        }
    });

    let spacer = Box::new(Orientation::Horizontal, 6);
    spacer.set_hexpand(true);

    let settings_btn = Button::from_icon_name("emblem-system-symbolic");
    settings_btn.add_css_class("icon-btn");

    let shortcuts_btn = Button::from_icon_name("starred-symbolic");
    shortcuts_btn.add_css_class("icon-btn");
    shortcuts_btn.set_tooltip_text(Some("List shortcuts"));

    let minimize_btn = Button::from_icon_name("window-minimize-symbolic");
    minimize_btn.add_css_class("icon-btn");

    let maximize_btn = Button::from_icon_name("view-fullscreen-symbolic");
    maximize_btn.add_css_class("icon-btn");

    let close_btn = Button::from_icon_name("window-close-symbolic");
    close_btn.add_css_class("icon-btn");

    header.append(&title);
    header.append(&time_label);
    header.append(&spacer);
    header.append(&shortcuts_btn);
    header.append(&settings_btn);
    header.append(&minimize_btn);
    header.append(&maximize_btn);
    header.append(&close_btn);

    {
        let win = window.clone();
        close_btn.connect_clicked(move |_| win.close());
    }

    {
        let win = window.clone();
        minimize_btn.connect_clicked(move |_| {
            win.minimize();
        });
    }

    {
        let win = window.clone();
        maximize_btn.connect_clicked(move |_| {
            if win.is_maximized() {
                win.unmaximize();
            } else {
                win.maximize();
            }
        });
    }

    // Drag to move when unlocked
    let win = window.clone();
    let settings_drag = settings.clone();
    let gesture = GestureClick::new();
    gesture.connect_pressed(move |g, _n, x, y| {
        if settings_drag.borrow().lock_in_place {
            return;
        }
        if let Some(surface) = win.surface() {
            let display = surface.display();
            if let Ok(toplevel) = surface.downcast::<gdk::Toplevel>() {
                if let Some(seat) = display.default_seat() {
                    if let Some(pointer) = seat.pointer() {
                        toplevel.begin_move(
                            &pointer,
                            g.current_button() as i32,
                            x,
                            y,
                            g.current_event_time(),
                        );
                    }
                }
            }
        }
    });
    header.add_controller(gesture);

    HeaderBar {
        widget: header,
        settings_btn,
        shortcuts_btn,
    }
}

fn create_standalone_window(app: &Application, title: &str, w: i32, h: i32) -> ApplicationWindow {
    let window = ApplicationWindow::new(app);
    window.set_title(Some(title));
    window.set_default_size(w, h);
    // window.set_decorated(false); // Optional: decide if we want OS decorations for standalone
    window.add_css_class("glass-window");
    
    // Attempt to detect compositor and apply appropriate classes
    if let Ok(session_type) = std::env::var("XDG_SESSION_TYPE") {
        if session_type == "wayland" {
             window.add_css_class("wayland-glass");
        } else {
             window.add_css_class("x11-glass");
        }
    }
    window
}

fn build_context_menu(handles: UiHandles) {
    let popover = Popover::builder().has_arrow(true).build();
    popover.set_parent(&handles.main_window);

    let column = Box::new(Orientation::Vertical, 8);
    column.set_margin_top(10);
    column.set_margin_bottom(10);
    column.set_margin_start(12);
    column.set_margin_end(12);

    let settings_btn = Button::with_label("Settings");
    settings_btn.add_css_class("pill-btn");
    column.append(&settings_btn);

    let lock_toggle = gtk4::CheckButton::with_label("Lock position");
    lock_toggle.set_active(handles.settings.borrow().lock_in_place);
    column.append(&lock_toggle);

    let size_toggle = gtk4::CheckButton::with_label("Lock size");
    size_toggle.set_active(handles.settings.borrow().lock_size);
    column.append(&size_toggle);

    let shortcuts_toggle = gtk4::CheckButton::with_label("Toggle shortcuts panel");
    shortcuts_toggle.set_active(handles.settings.borrow().show_shortcuts_panel);
    column.append(&shortcuts_toggle);

    let opacity_box = Box::new(Orientation::Vertical, 4);
    opacity_box.append(&Label::new(Some("Opacity")));
    let opacity_scale = gtk4::Scale::with_range(Orientation::Horizontal, 0.1, 1.0, 0.05);
    opacity_scale.set_value(1.0); // Default, TODO: bind to settings
    opacity_scale.set_draw_value(true);
    opacity_box.append(&opacity_scale);
    column.append(&opacity_box);

    {
        let win = handles.main_window.clone();
        opacity_scale.connect_value_changed(move |scale| {
            win.set_opacity(scale.value());
        });
    }

    let minimize_btn = Button::with_label("Minimize");
    let close_btn = Button::with_label("Close");
    close_btn.add_css_class("danger");
    column.append(&minimize_btn);
    column.append(&close_btn);

    popover.set_child(Some(&column));

    let gesture = GestureClick::new();
    gesture.set_button(3);
    let pop_clone = popover.clone();
    gesture.connect_pressed(move |_, _, x, y| {
        pop_clone.set_pointing_to(Some(&gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
        pop_clone.popup();
    });
    handles.main_window.add_controller(gesture);

    {
        let win = handles.main_window.clone();
        let pop = popover.clone();
        minimize_btn.connect_clicked(move |_| {
            win.minimize();
            pop.popdown();
        });
    }

    {
        let win = handles.main_window.clone();
        close_btn.connect_clicked(move |_| {
            win.close();
        });
    }

    {
        let handles_clone = handles.clone();
        let pop = popover.clone();
        settings_btn.connect_clicked(move |_| {
            let settings_rc = handles_clone.settings.clone();
            let handles_apply = handles_clone.clone();
            show_settings_window(
                &handles_clone.main_window,
                settings_rc,
                move |updated: Settings| {
                    handles_apply.settings.replace(updated.clone());
                    apply_settings(&handles_apply, &updated);
                },
            );
            pop.popdown();
        });
    }

    {
        let handles_clone = handles.clone();
        lock_toggle.connect_toggled(move |btn| {
            let mut s = handles_clone.settings.borrow_mut();
            s.lock_in_place = btn.is_active();
            s.save();
            let snapshot = s.clone();
            drop(s);
            apply_settings(&handles_clone, &snapshot);
        });
    }

    {
        let handles_clone = handles.clone();
        size_toggle.connect_toggled(move |btn| {
            let mut s = handles_clone.settings.borrow_mut();
            s.lock_size = btn.is_active();
            s.save();
            let snapshot = s.clone();
            drop(s);
            apply_settings(&handles_clone, &snapshot);
        });
    }

    {
        let handles_clone = handles.clone();
        shortcuts_toggle.connect_toggled(move |btn| {
            let mut s = handles_clone.settings.borrow_mut();
            s.show_shortcuts_panel = btn.is_active();
            s.save();
            let snapshot = s.clone();
            drop(s);
            apply_settings(&handles_clone, &snapshot);
        });
    }
}
