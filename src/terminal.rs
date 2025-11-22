use gtk4::prelude::*;
use gtk4::{gio, glib};
use vte4::{PtyFlags, Terminal, TerminalExtManual};

pub fn create_terminal(shell: &str, cwd: Option<&str>, env: Option<&[(&str, &str)]>) -> Terminal {
    let terminal = Terminal::new();
    terminal.add_css_class("glass-terminal");

    let command = [shell];

    let env_vars: Vec<String> = if let Some(vars) = env {
        vars.iter().map(|(k, v)| format!("{}={}", k, v)).collect()
    } else {
        Vec::new()
    };
    let env_ptrs: Vec<&str> = env_vars.iter().map(|s| s.as_str()).collect();

    terminal.spawn_async(
        PtyFlags::DEFAULT,
        cwd,
        &command,
        &env_ptrs,
        glib::SpawnFlags::DEFAULT,
        || {},                     // child setup closure
        -1,                        // timeout
        None::<&gio::Cancellable>, // cancellable
        |_| {},                    // callback
    );

    terminal
}
