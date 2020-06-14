// {{{ Crate docs
//! Telegraf `Drain` for `slog-rs`
//!
//! ```
//!use slog::{Logger, Drain, o, info};
//!use slog_telegraf::{TelegrafDrain};
//!
//!fn main() {
//!    let drain = TelegrafDrain::new("tcp://192.168.0.108:8094".into(), "measurement".into()).unwrap().fuse();
//!    let drain = slog_async::Async::new(drain).build().fuse();
//!
//!    let log = Logger::root(drain, o!("ver" => "1.2.1"));
//!    info!(log, "log"; "field_key" => 10);
//!}
//! ```
// }}}


#[cfg_attr(test, macro_use)]
extern crate slog;
extern crate url;

mod drain;
mod error;
mod ser;
mod telegraf;

pub use drain::{TelegrafDrain, TelegrafDrainBuilder};
pub use error::Error;
pub use telegraf::{Client};
pub use ser::TelegrafSocketSerializer;
