use gtk4::prelude::*;
use gtk4::{gio, glib};
use vte4::{PtyFlags, Terminal, TerminalExtManual};

pub fn create_terminal() -> Terminal {
    let terminal = Terminal::new();
    terminal.add_css_class("glass-terminal");

    let shell = std::env::var("SHELL").unwrap_or("/bin/bash".to_string());
    let command = [shell.as_str()];

    terminal.spawn_async(
        PtyFlags::DEFAULT,
        None, // working directory
        &command,
        &[], // env
        glib::SpawnFlags::DEFAULT,
        || {},                     // child setup closure
        -1,                        // timeout
        None::<&gio::Cancellable>, // cancellable
        |_| {},                    // callback
    );

    terminal
}
