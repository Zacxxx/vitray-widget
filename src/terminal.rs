use gtk4::prelude::*;
use gtk4::{glib, Align, Button, Box, Orientation, Label};

#[cfg(target_os = "linux")]
use vte4::{PtyFlags, Terminal, TerminalExtManual};

use crate::platform;

pub fn create_terminal(shell: &str, cwd: Option<&str>, env: Option<&[(&str, &str)]>) -> gtk4::Widget {
    #[cfg(target_os = "linux")]
    {
        let terminal = Terminal::new();
        terminal.add_css_class("glass-terminal");

        let command = [shell];

        let env_vars: Vec<String> = env.map_or_else(Vec::new, |vars| {
            vars.iter().map(|(k, v)| format!("{k}={v}")).collect()
        });
        let env_ptrs: Vec<&str> = env_vars.iter().map(String::as_str).collect();

        terminal.spawn_async(
            PtyFlags::DEFAULT,
            cwd,
            &command,
            &env_ptrs,
            glib::SpawnFlags::DEFAULT,
            || {},                     // child setup closure
            -1,                        // timeout
            None::<&gtk4::gio::Cancellable>, // cancellable
            |_| {},                    // callback
        );

        terminal.upcast::<gtk4::Widget>()
    }

    #[cfg(not(target_os = "linux"))]
    {
        let container = Box::new(Orientation::Vertical, 12);
        container.set_valign(Align::Center);
        container.set_halign(Align::Center);
        container.add_css_class("glass-terminal-placeholder");

        let label = Label::new(Some("Terminal not available in embedded mode."));
        label.add_css_class("terminal-placeholder-text");
        
        let button = Button::with_label("Launch External Terminal");
        button.add_css_class("terminal-launch-button");
        
        button.connect_clicked(|_| {
            platform::open_external_terminal(None);
        });

        let warp_button = Button::with_label("Launch Warp");
        warp_button.add_css_class("terminal-launch-button");
        warp_button.connect_clicked(|_| {
            platform::open_external_terminal(Some("warp"));
        });

        container.append(&label);
        container.append(&button);
        container.append(&warp_button);

        container.upcast::<gtk4::Widget>()
    }
}
