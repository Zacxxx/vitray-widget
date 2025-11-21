use gtk4::gdk::prelude::*;
use gtk4::prelude::*;
use gtk4::{
    gdk, glib, Align, Application, ApplicationWindow, Box, Button, CssProvider, DrawingArea,
    GestureClick, Grid, Label, LevelBar, Notebook, Orientation, Popover, Separator, Stack,
    StackTransitionType,
};
use std::{cell::RefCell, rc::Rc};
use vte4::TerminalExt;

use crate::monitor::SystemMonitor;
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
    window: ApplicationWindow,
    root: Box,
    terminal_section: Box,
    monitoring_section: Box,
    monitor_cards: MonitorGroup,
    performance_strip: PerformanceStrip,
    shortcuts_panel: ShortcutsPanel,
    settings: Rc<RefCell<Settings>>,
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

    let window = ApplicationWindow::new(app);
    window.set_title(Some("Vitray Widget"));
    window.set_default_size(520, 760);
    window.set_decorated(false);
    
    // Attempt to detect compositor and apply appropriate classes
    if let Ok(session_type) = std::env::var("XDG_SESSION_TYPE") {
        if session_type == "wayland" {
             window.add_css_class("wayland-glass");
        } else {
             window.add_css_class("x11-glass");
        }
    }
    window.add_css_class("glass-window");
    window.set_resizable(!settings.borrow().lock_size);

    // --- Terminal channel ---
    #[allow(deprecated)]
    let (sender, receiver) = ::glib::MainContext::channel(::glib::Priority::DEFAULT);

    // --- Root layout ---
    let root = Box::new(Orientation::Horizontal, 10);
    root.add_css_class("root");
    root.set_margin_top(10);
    root.set_margin_bottom(10);
    root.set_margin_start(10);
    root.set_margin_end(10);

    let main_column = Box::new(Orientation::Vertical, 12);
    main_column.add_css_class("glass-panel");

    // --- Header ---
    let performance_strip = PerformanceStrip::new();
    let header = build_header(&window, settings.clone());
    main_column.append(&header.widget);
    main_column.append(&performance_strip.widget());

    // --- Terminal Section ---
    let terminal_section = Box::new(Orientation::Vertical, 6);
    terminal_section.add_css_class("terminal-section");

    let terminal_header = Box::new(Orientation::Horizontal, 8);
    terminal_header.add_css_class("terminal-header");
    let tabs_btn = Button::with_label("+ Tab");
    tabs_btn.add_css_class("pill-btn");
    
    // Enable reordering
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

    // Notebook created above
    // notebook.set_show_tabs(true); // Already set
    // notebook.set_scrollable(true); // Already set
    // TODO(senior-ui): Persist tabs + working directories so a reboot restores the same workspace.

    let terminal = create_terminal(None, None);
    // TODO(senior-ui): Allow each page to bind to saved shortcuts (Deploy, Monitor) instead of
    // generic names like T1/T2.
    let scrolled = gtk4::ScrolledWindow::new();
    scrolled.set_child(Some(&terminal));
    scrolled.set_vexpand(true);
    
    let tab_label = build_tab_label(&notebook, &scrolled, "T1");
    notebook.append_page(&scrolled, Some(&tab_label));
    notebook.set_tab_reorderable(&scrolled, true);
    notebook.set_tab_detachable(&scrolled, true);

    terminal_section.append(&terminal_header);
    terminal_section.append(&notebook);

    // --- Middle separator ---
    let separator = Separator::new(Orientation::Horizontal);
    separator.add_css_class("section-separator");

    // --- Monitoring ---
    let monitoring_section = Box::new(Orientation::Vertical, 8);
    monitoring_section.add_css_class("monitoring-shell");
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
    // TODO(senior-ui): Let users reorder / collapse these cards and expose additional sensors
    // (swap, temps, battery) pulled from sysfs to tailor the dashboard.

    grid.attach(&monitor_cards.cpu.container, 0, 0, 1, 1);
    grid.attach(&monitor_cards.gpu.container, 1, 0, 1, 1);
    grid.attach(&monitor_cards.ram.container, 0, 1, 1, 1);
    grid.attach(&monitor_cards.net.container, 1, 1, 1, 1);
    monitoring_section.append(&grid);

    main_column.append(&terminal_section);
    main_column.append(&separator);
    main_column.append(&monitoring_section);

    // --- Shortcuts side panel ---
    let shortcuts_panel = ShortcutsPanel::new(&window, sender.clone());
    // TODO(senior-ui): Animate panel width responsively and snap it to the opposite edge on RTL
    // locales so it doesn't fight terminal space on ultra-wide screens.
    shortcuts_panel.set_revealed(settings.borrow().show_shortcuts_panel);

    root.append(&main_column);
    let gap = Separator::new(Orientation::Vertical);
    gap.add_css_class("side-gap");
    root.append(&gap);
    root.append(&shortcuts_panel.revealer);

    window.set_child(Some(&root));

    // Terminal channel feed
    receiver.attach(None, move |cmd: String| {
        let cmd_with_newline = format!("{}\n", cmd);
        terminal.feed_child(cmd_with_newline.as_bytes());
        ::glib::ControlFlow::Continue
    });

    {
        let notebook_clone = notebook.clone();
        tabs_btn.connect_clicked(move |_| add_terminal_tab(&notebook_clone));
    }

    {
        let panel_clone = shortcuts_panel.clone();
        shortcuts_btn.connect_clicked(move |_| panel_clone.toggle());
    }

    let handles = UiHandles {
        window: window.clone(),
        root: root.clone(),
        terminal_section: terminal_section.clone(),
        monitoring_section: monitoring_section.clone(),
        monitor_cards: monitor_cards.clone(),
        performance_strip: performance_strip.clone(),
        shortcuts_panel: shortcuts_panel.clone(),
        settings: settings.clone(),
    };

    // Context menu / right click
    build_context_menu(handles.clone());

    {
        let handles_clone = handles.clone();
        header.settings_btn.connect_clicked(move |_| {
            let settings_rc = handles_clone.settings.clone();
            let handles_apply = handles_clone.clone();
            show_settings_window(
                &handles_clone.window,
                settings_rc,
                move |updated: Settings| {
                    handles_apply.settings.replace(updated.clone());
                    apply_settings(&handles_apply, &updated);
                },
            );
        });
    }

    {
        let panel_clone = handles.shortcuts_panel.clone();
        header
            .shortcuts_btn
            .connect_clicked(move |_| panel_clone.toggle());
    }
    apply_settings(&handles, &settings.borrow());

    window.present();

    // --- Update Loop ---
    let monitor = Rc::new(RefCell::new(SystemMonitor::new()));
    let mut last_net: Option<(u64, u64)> = None;

    glib::timeout_add_seconds_local(1, move || {
        let mut mon = monitor.borrow_mut();
        mon.refresh();
        let active_settings = handles.settings.borrow().clone();
        let style = active_settings.monitor_style.clone();

        let cpu = mon.get_cpu_usage() as f64;
        handles
            .monitor_cards
            .cpu
            .update(cpu, &format!("{cpu:.0}%"), &style);

        let gpu_usage = mon.get_gpu_usage();
        // TODO(senior-ui): Detect when GPUs go to sleep and slow the polling rate to cut power use.
        if let Some(gpu) = gpu_usage {
            handles
                .monitor_cards
                .gpu
                .update(gpu as f64, &format!("{gpu:.0}%"), &style);
        } else {
            handles.monitor_cards.gpu.update(0.0, "N/A", &style);
        }

        let (used, total) = mon.get_ram_usage();
        let used_gb = used as f64 / 1024.0 / 1024.0 / 1024.0;
        let total_gb = total as f64 / 1024.0 / 1024.0 / 1024.0;
        let ram_pct = if total > 0 {
            used as f64 / total as f64 * 100.0
        } else {
            0.0
        };
        handles.monitor_cards.ram.update(
            ram_pct,
            &format!("{used_gb:.1}/{total_gb:.1} GB"),
            &style,
        );

        let (rx_raw, tx_raw) = mon.get_network_stats();
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

        handles.monitor_cards.net.update(
            total_speed.min(2000.0),
            &format!("↓{} ↑{}", rx_display, tx_display),
            &style,
        );

        let gpu_label = gpu_usage.map(|gpu| format!("GPU {:.0}%", gpu));
        let net_label = format!("NET {total_speed:.0} KB/s");

        handles.performance_strip.update(
            &format!("CPU {cpu:.0}%"),
            gpu_label.as_deref().unwrap_or("GPU —"),
            &format!("RAM {ram_pct:.0}%"),
            &net_label,
        );

        glib::ControlFlow::Continue
    });
}

fn add_terminal_tab(notebook: &Notebook) {
    let terminal = create_terminal(None, None);
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
    apply_theme(&handles.root, &settings.theme);
    handles.terminal_section.set_visible(settings.show_terminal);
    handles
        .monitoring_section
        .set_visible(settings.show_monitoring);
    handles.monitor_cards.set_visibility(settings);
    handles
        .shortcuts_panel
        .set_revealed(settings.show_shortcuts_panel);
    handles.window.set_resizable(!settings.lock_size);
    handles.monitor_cards.set_style(&settings.monitor_style);
}

fn apply_theme(root: &Box, theme: &Theme) {
    for cls in &[
        "theme-dark",
        "theme-light",
        "theme-solarized",
        "theme-tokyo",
    ] {
        root.remove_css_class(cls);
    }
    let class_name = match theme {
        Theme::Dark => "theme-dark",
        Theme::Light => "theme-light",
        Theme::Solarized => "theme-solarized",
        Theme::Tokyo => "theme-tokyo",
    };
    // TODO(senior-ui): Allow custom accent colors + transparency slider instead of fixed presets.
    root.add_css_class(class_name);
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

fn build_context_menu(handles: UiHandles) {
    let popover = Popover::builder().has_arrow(true).build();
    popover.set_parent(&handles.window);

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
        let win = handles.window.clone();
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
    handles.window.add_controller(gesture);

    {
        let win = handles.window.clone();
        let pop = popover.clone();
        minimize_btn.connect_clicked(move |_| {
            win.minimize();
            pop.popdown();
        });
    }

    {
        let win = handles.window.clone();
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
                &handles_clone.window,
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
