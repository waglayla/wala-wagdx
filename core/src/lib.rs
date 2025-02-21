#![warn(clippy::all, rust_2018_idioms)]

extern crate self as wala_wagdx_core;

pub mod core;
pub use core::Core;

pub mod dx_manager;

pub mod gui;
pub mod app;

pub mod utils;
pub mod settings;
pub mod imports;
pub mod components;
mod result;
pub use result::*;

mod dx_wallet;
pub use dx_wallet::*;

pub mod error;
pub mod events;
pub mod storage;
pub mod fonts;
pub mod network;
pub mod collection;
pub mod node_state;

pub mod assets;
pub mod platform;