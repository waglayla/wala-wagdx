#![warn(clippy::all, rust_2018_idioms)]

extern crate self as wala_wagdx_core;

pub mod core;
pub use core::Core;

pub mod dx_manager;

pub mod gui;
pub mod app;

pub mod utils;
pub mod settings;
pub mod frame;
pub mod imports;
pub mod components;
// pub mod interop;
pub mod result;
pub mod error;
pub mod storage;
pub mod fonts;