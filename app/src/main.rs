mod app;
mod route;
mod theme;
mod i18n;
mod ui;
mod features;

fn main() {
    dioxus::launch(app::App);
}
