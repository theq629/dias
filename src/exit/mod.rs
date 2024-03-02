//! Support for exiting programs.
//!
//! - Standard: uses a wrapper for [std::process::exit()].
//! - Web: not supported.
//!
//! ```rust
//! use dias::exit::{make_exiter, Exiter};
//!
//! make_exiter().unwrap().exit();
//! ```

mod generic;

#[cfg(not(target_arch = "wasm32"))]
mod standard;

#[cfg(target_arch = "wasm32")]
mod dummy;

pub use generic::Exiter;

use crate::AvailabilityError;

pub fn make_exiter() -> Result<impl Exiter, AvailabilityError> {
    #[cfg(not(target_arch = "wasm32"))]
    return Ok(standard::Exiter::new());
    #[cfg(target_arch = "wasm32")]
    Err::<dummy::DummyExiter, _>(AvailabilityError::NotSupported)
}
