mod api_info;
mod gen_api;
mod gen_api_sys;
mod install;
mod main_impl;

type Error = Box<dyn std::error::Error + 'static>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    main_impl::run().await
}
