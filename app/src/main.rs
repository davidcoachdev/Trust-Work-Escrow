mod app;
mod route;
mod theme;
mod i18n;
mod ui;
mod features;
mod server;

fn main() {
    dioxus::launch(app::App);
}
