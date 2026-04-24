mod app;
mod bridge;
mod components;
mod pages;
mod tokens;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(app::App);
}
