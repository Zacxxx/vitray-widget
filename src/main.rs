//! Vitray widget application entry point.
//!
//! This crate provides a desktop widget with system monitoring, terminal, and shortcuts.

use crate::shortcuts::Shortcuts;
use crate::ui::build_ui;
use clap::{ArgAction, Parser};
use gtk4::prelude::*;
use gtk4::Application;
use std::process::Command;

mod gpu;
mod monitor;
mod settings;
mod settings_ui;
mod shortcuts;
mod shortcuts_ui;
mod terminal;
mod ui;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Vitray widget: glassy terminal + performance HUD.",
    long_about = "Vitray widget: a glassy terminal with performance monitoring, shortcuts, and themes.",
    after_help = "Examples:\n  vitray --shortcut \"htop\" \"Monitor\"\n  vitray --remove-shortcut \"Monitor\"\n  vitray --list-shortcuts\n  vitray deploy   # runs saved shortcut named 'deploy'"
)]
struct Args {
    /// Add a new shortcut: vitray --shortcut "command" "name"
    #[arg(long, num_args = 2, value_names = ["COMMAND", "NAME"])]
    shortcut: Option<Vec<String>>,

    /// Remove a shortcut by name
    #[arg(long, value_name = "NAME")]
    remove_shortcut: Option<String>,

    /// List saved shortcuts
    #[arg(long, action = ArgAction::SetTrue)]
    list_shortcuts: bool,

    /// Run a saved shortcut: vitray --run "name"
    #[arg(long, value_name = "NAME")]
    run: Option<String>,

    /// Run a saved shortcut directly: vitray <name>
    #[arg(value_name = "SHORTCUT")]
    shortcut_name: Option<String>,
}

#[allow(clippy::print_stdout)]
fn main() {
    let args = Args::parse();

    if let Some(shortcut_args) = args.shortcut {
        if shortcut_args.len() == 2 {
            let command = &shortcut_args[0];
            let name = &shortcut_args[1];
            let mut shortcuts = Shortcuts::load();
            match shortcuts.add(name, command.clone()) {
                Ok(()) => println!("Shortcut '{name}' added for command '{command}'"),
                Err(e) => eprintln!("Error adding shortcut: {e}"),
            }
            return;
        }
    }

    if let Some(name) = args.remove_shortcut {
        let mut shortcuts = Shortcuts::load();
        if shortcuts.remove_by_name(&name) {
            println!("Removed shortcut '{name}'");
        } else {
            eprintln!("Shortcut '{name}' not found");
        }
        return;
    }

    if args.list_shortcuts {
        let shortcuts = Shortcuts::load();
        if shortcuts.items.is_empty() {
            println!("No shortcuts defined. Add one with --shortcut \"command\" \"name\".");
        } else {
            println!("Saved shortcuts:");
            for s in shortcuts.items {
                println!("- {} :: {}", s.name, s.command);
            }
        }
        return;
    }

    if let Some(name) = args.run.or(args.shortcut_name) {
        let shortcuts = Shortcuts::load();
        if let Some(shortcut) = shortcuts.find(&name) {
            println!("→ Running {} :: {}", shortcut.name, shortcut.command);
            let _ = Command::new("bash")
                .arg("-c")
                .arg(shortcut.command)
                .status();
        } else {
            eprintln!(
                "Shortcut '{name}' not found. Use --list-shortcuts to see available entries."
            );
        }
        return;
    }

    let app = Application::builder()
        .application_id("com.moebius.vitray-widget")
        .build();

    app.connect_startup(|_| {
        if let Ok(_icon_path) = std::fs::canonicalize("assets/icon.png") {
             gtk4::Window::set_default_icon_name("vitray-widget");
        }
    });

    // TODO(senior-ui): Register a single-instance DBus name and raise the existing window when
    // invoked again instead of spawning duplicate widgets.

    app.connect_activate(|app| {
        build_ui(app);
    });

    app.run();
}
