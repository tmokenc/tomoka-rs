#![allow(clippy::unreadable_literal)]

#[cfg(feature = "ancient_magic")]
pub mod ancient_magic;
#[cfg(feature = "dark_magic")]
pub mod dark_magic;
#[cfg(feature = "image_magic")]
pub mod image;
#[cfg(feature = "macro_magic")]
pub mod macros;
#[cfg(feature = "sauce_magic")]
pub mod sauce;
#[cfg(feature = "trait_magic")]
pub mod traits;

#[cfg(feature = "dark_magic")]
pub use dark_magic::*;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
