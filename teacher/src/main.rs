#![recursion_limit = "128"]
extern crate bincode;
use stdweb::Value;
#[macro_use]
extern crate serde_derive;
extern crate bzip2;
extern crate serde;

#[macro_use]
extern crate stdweb;
mod platform;
mod teacher;
use stdweb::unstable::TryInto;
mod pdollarplus;

include!("mmp.rs");

fn main() {
    stdweb::initialize();
    platform_init();
    teacher::init();
    stdweb::event_loop();
}
