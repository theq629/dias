//! Command line options.
//!
//! - Standard: uses command line arguments.
//! - Web: uses URL parameters.
//!
//! Note that this is currently strictly options, no required or positional arguments.
//!
//! ```rust
//! use dias::cmd_line::{make_cmd_line_parser, Parser, Parsed};
//!
//! let mut parser = make_cmd_line_parser().unwrap();
//! let foo = parser.add_flag(&['f'], &["foo"]);
//! let bar = parser.add_option::<String, _>(&['b'], &["bar"]);
//! let parsed = parser.parse().unwrap();
//! let _ = parsed.get(&foo);
//! let _ = parsed.get(&bar);
//! ```

mod dummy;
mod generic;
mod shared;

#[cfg(not(target_arch = "wasm32"))]
mod standard;
#[cfg(target_arch = "wasm32")]
mod web;

pub use dummy::{DummyParsed, DummyParser};
pub use generic::{Parsed, Parser, ParsingError};

use crate::AvailabilityError;

pub fn make_cmd_line_parser() -> Result<impl Parser, AvailabilityError> {
    #[cfg(not(target_arch = "wasm32"))]
    return Ok(standard::Parser::new());
    #[cfg(target_arch = "wasm32")]
    return Ok(web::Parser::new());
}
