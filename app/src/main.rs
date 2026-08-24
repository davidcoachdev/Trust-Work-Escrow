mod app;
mod route;
mod theme;
mod i18n;
mod ui;
mod features;
mod server;

#[cfg(feature = "server")]
#[tokio::main]
async fn main() {
    use dioxus_server::DioxusRouterExt;

    let address = dioxus::cli_config::fullstack_address_or_localhost();
    let router = axum::Router::<dioxus_server::FullstackState>::new()
        .serve_dioxus_application(dioxus_server::ServeConfig::new(), app::App);
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("failed to bind Dioxus server");

    axum::serve(listener, router.into_make_service())
        .await
        .expect("Dioxus server stopped unexpectedly");
}

#[cfg(not(feature = "server"))]
fn main() {
    dioxus::launch(app::App);
}
