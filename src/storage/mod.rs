//! File-like storage.
//!
//! - Standard: uses files. Standard locations are provided by the [directories] crate and should
//!   cover desktop XDG, macOS, and Windows standards.
//! - Web: uses web storage.
//!
//! ```rust
//! use std::io::{Read, Write};
//! use dias::storage::{make_storage, Storage, Dir, File, WritableDir, WritableFile};
//!
//! let mut file = make_storage("Bar App", Some("Foo Corp"), Some("com"))
//!     .unwrap()
//!     .writable_data()
//!     .unwrap()
//!     .writable_file("test".into());
//! write!(file.write_text().unwrap(), "hello world");
//! let mut read = String::new();
//! file.read_text().unwrap().read_to_string(&mut read).unwrap();
//! ```

pub mod boxable;
mod generic;
mod memory;

#[cfg(not(target_arch = "wasm32"))]
mod standard;
#[cfg(target_arch = "wasm32")]
mod web;

use crate::AvailabilityError;
use std::error::Error;

pub use generic::{
    Dir, File, OuterDirectoryError, ParentDir, Storage, WritableDir, WritableFile,
    WritableParentDir,
};
pub use memory::MemoryStorage;

/// Get a storage object if available.
///
/// The arguments are for uniqueness on various platforms.
///
/// - `application` - The name of the application. Can contain whitespace and does not need to be
///   lowercase.
/// - `organization` - The name of the organization developing the application, or empty if not
///   applicable. Can contain whitespace and does not need to be lowercase. Set `None` if not
///   applicable.
/// - `qualifier` - The reverse domain name notation of the application, excluding the organization
///   or application name. Set `None` if not applicable.
pub fn make_storage(
    application: &str,
    organization: Option<&str>,
    qualifier: Option<&str>,
) -> Result<impl Storage, AvailabilityError> {
    let qualifier = qualifier.unwrap_or("");
    let organization = organization.unwrap_or("");
    let _ = (qualifier, organization, application);
    #[cfg(not(target_arch = "wasm32"))]
    return standard::Storage::new(qualifier, organization, application)
        .map_err(|e| AvailabilityError::NotAvailable(Some(Box::new(e) as Box<dyn Error>)));
    #[cfg(target_arch = "wasm32")]
    return web::Storage::new()
        .map_err(|e| AvailabilityError::NotAvailable(Some(Box::new(e) as Box<dyn Error>)));
}
