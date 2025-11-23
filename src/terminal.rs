use gtk4::prelude::*;
use gtk4::{glib, Align, Label};

#[cfg(target_os = "linux")]
use vte4::{PtyFlags, Terminal, TerminalExtManual};

#[derive(Clone)]
pub enum AppTerminal {
    #[cfg(target_os = "linux")]
    Linux(Terminal),
    #[cfg(not(target_os = "linux"))]
    Windows(Label),
}

impl AppTerminal {
    pub fn widget(&self) -> &impl IsA<gtk4::Widget> {
        match self {
            #[cfg(target_os = "linux")]
            Self::Linux(t) => t,
            #[cfg(not(target_os = "linux"))]
            Self::Windows(l) => l,
        }
    }

    pub fn feed_child(&self, text: &[u8]) {
        match self {
            #[cfg(target_os = "linux")]
            Self::Linux(t) => t.feed_child(text),
            #[cfg(not(target_os = "linux"))]
            Self::Windows(_) => {},
        }
    }
}

pub fn create_terminal(shell: &str, cwd: Option<&str>, env: Option<&[(&str, &str)]>) -> AppTerminal {
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

        AppTerminal::Linux(terminal)
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = shell;
        let _ = cwd;
        let _ = env;
        let label = Label::new(Some("Terminal not supported on Windows"));
        label.add_css_class("glass-terminal");
        label.set_halign(Align::Center);
        label.set_valign(Align::Center);
        AppTerminal::Windows(label)
    }
}
