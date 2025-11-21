use gtk4::prelude::*;
use gtk4::Application;
use clap::Parser;
use crate::ui::build_ui;
use crate::shortcuts::Shortcuts;

mod monitor;
mod terminal;
mod ui;
mod settings;
mod shortcuts;
mod gpu;
mod settings_ui;
mod shortcuts_ui;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Add a new shortcut: vitray --shortcut "command" "name"
    #[arg(long, num_args = 2, value_names = ["COMMAND", "NAME"])]
    shortcut: Option<Vec<String>>,
}

fn main() {
    let args = Args::parse();

    if let Some(shortcut_args) = args.shortcut {
        if shortcut_args.len() == 2 {
            let command = &shortcut_args[0];
            let name = &shortcut_args[1];
            let mut shortcuts = Shortcuts::load();
            shortcuts.add(name.clone(), command.clone());
            println!("Shortcut '{}' added for command '{}'", name, command);
            return;
        }
    }

    let app = Application::new(
        Some("com.moebius.vitray-widget"),
        Default::default(),
    );

    app.connect_activate(|app| {
        build_ui(app);
    });

    app.run();
}
