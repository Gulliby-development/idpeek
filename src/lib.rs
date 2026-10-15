//! Public modules and injectable entry point for the idpeek library.

pub mod args;
pub mod base32;
pub mod calendar;
pub mod cli;
pub mod detect;
pub mod ksuid;
pub mod objectid;
pub mod report;
pub mod snowflake;
pub mod text;
pub mod ulid;
pub mod uuid;

pub use cli::run;
