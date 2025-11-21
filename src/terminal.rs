use vte4::{Terminal, TerminalExtManual, PtyFlags};
use gtk4::{glib, gio};

pub fn create_terminal() -> Terminal {
    let terminal = Terminal::new();
    
    let shell = std::env::var("SHELL").unwrap_or("/bin/bash".to_string());
    let command = [shell.as_str()];
    
    terminal.spawn_async(
        PtyFlags::DEFAULT,
        None, // working directory
        &command,
        &[], // env
        glib::SpawnFlags::DEFAULT,
        || {}, // child setup closure
        -1,   // timeout
        None::<&gio::Cancellable>, // cancellable
        |_| {} // callback
    );

    terminal
}
