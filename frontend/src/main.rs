use gloo_console::log;

mod app;
mod components;
mod api;

use app::App;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    log!("Starting Gillean Blockchain Frontend");
    yew::Renderer::<App>::new().render();
}
