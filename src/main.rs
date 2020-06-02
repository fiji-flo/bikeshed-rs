#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate markup5ever;
#[macro_use]
extern crate maplit;

#[macro_use]
mod util;
mod boilerplate;
mod clean;
mod client;
mod config;
mod html;
mod line;
mod metadata;
mod spec;

fn main() {
    client::run();
}
