use relm4::prelude::*;

mod app;
mod components;
mod database;
mod i18n;
mod icon;
mod utils;

use app::AppModel;

use gtk4::prelude::*;
use gtk4::gio;

fn main() {
    let app = gtk4::Application::builder()
        .application_id("com.arianpg.civiewer")
        .flags(gio::ApplicationFlags::HANDLES_COMMAND_LINE | gio::ApplicationFlags::NON_UNIQUE)
        .build();

    app.connect_command_line(|app, _cmd| {
        app.activate();
        0
    });

    let relm = RelmApp::from_app(app);
    relm.run::<AppModel>(());
}
mod input_settings;
