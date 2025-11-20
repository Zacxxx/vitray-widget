use gtk4::prelude::*;
use gtk4::Application;
use crate::ui::build_ui;

mod monitor;
mod terminal;
mod ui;

fn main() {
    let app = Application::new(
        Some("com.moebius.vitray-widget"),
        Default::default(),
    );

    app.connect_activate(|app| {
        build_ui(app);
    });

    app.run();
}
