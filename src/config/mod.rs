//! Config file loading and saving.
//!
//! - Standard: uses TOML.
//! - Web: uses JSON.
//!
//! ```rust
//! use std::io::Cursor;
//! use serde::{Deserialize, Serialize};
//! use dias::config::{read_config, write_config};
//!
//! #[derive(Serialize, Deserialize)]
//! pub struct TestConfig {
//!     pub foo: i32,
//!     pub bar: String,
//! }
//! let config = TestConfig {
//!     foo: 12345,
//!     bar: "hello world".to_string(),
//! };
//! let mut buf = Cursor::new(Vec::new()); // probably you want a real file instead
//! write_config(&config, &mut buf).unwrap();
//! buf.set_position(0);
//! let _: TestConfig = read_config(&mut buf).unwrap();
//! ```

mod generic;

#[cfg(not(target_arch = "wasm32"))]
mod standard;
#[cfg(target_arch = "wasm32")]
mod web;

use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

use generic::ConfigStringHandler as _;
#[cfg(not(target_arch = "wasm32"))]
use standard::ConfigStringHandler;
#[cfg(target_arch = "wasm32")]
use web::ConfigStringHandler;

pub fn write_config<T, W>(config: &T, write: &mut W) -> std::io::Result<()>
where
    T: Serialize,
    W: Write,
{
    let out = ConfigStringHandler::to_string(&config).map_err(std::io::Error::other)?;
    write!(write, "{}", out)?;
    Ok(())
}

pub fn read_config<T, R>(read: &mut R) -> std::io::Result<T>
where
    T: for<'a> Deserialize<'a>,
    R: Read,
{
    let input = std::io::read_to_string(read)?;
    let config = ConfigStringHandler::from_str(&input).map_err(std::io::Error::other)?;
    Ok(config)
}

#[cfg(feature = "storage")]
pub fn write_config_file<T>(
    config: &T,
    file: &mut impl crate::storage::WritableFile,
) -> std::io::Result<()>
where
    T: Serialize,
{
    write_config(config, &mut file.write_text()?)
}

#[cfg(feature = "storage")]
pub fn read_config_file<T>(file: &impl crate::storage::File) -> std::io::Result<T>
where
    T: for<'a> Deserialize<'a>,
{
    read_config(&mut file.read_text()?)
}
