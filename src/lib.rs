//! Minimal cross-platform support for common platform specific things, intended for small games
//! for web plus desktopy platforms.
//!
//! Here "standard" refers to the case where [std] is available and working (ie not web).
//!
//! Currently tested on Linux and Web. In principle should work fine on Mac OS X and Windows.

mod availability;

#[cfg(feature = "exit")]
#[cfg_attr(docsrs, doc(cfg(feature = "exit")))]
pub mod exit;

#[cfg(feature = "storage")]
#[cfg_attr(docsrs, doc(cfg(feature = "storage")))]
pub mod storage;

#[cfg(feature = "cmd-line")]
#[cfg_attr(docsrs, doc(cfg(feature = "cmd-line")))]
pub mod cmd_line;

#[cfg(feature = "config")]
#[cfg_attr(docsrs, doc(cfg(feature = "config")))]
pub mod config;

pub use availability::AvailabilityError;
