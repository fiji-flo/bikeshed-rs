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
mod fix;
mod heading;
mod html;
mod line;
mod markdown;
mod metadata;
mod shorthand;
mod spec;
#[cfg(test)]
mod test;

fn main() {
    client::run();
}
