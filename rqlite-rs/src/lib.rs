#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("lib.md")]
#![warn(clippy::pedantic, clippy::all)]
#![allow(clippy::module_name_repetitions, clippy::similar_names)]

pub mod client;
pub mod query;
pub mod query_result;
pub mod response;
pub use client::{RqliteClient, RqliteClientBuilder};
pub use rqlite_rs_core::*;
pub mod batch;
pub mod config;
pub mod error;
pub mod node;
pub mod request;
pub mod fallback;
pub(crate) mod select;

#[cfg(feature = "macros")]
pub use rqlite_rs_macros::*;

pub use rqlite_rs_core::decode;

pub mod prelude {
    pub use crate::client::RqliteClient;
    pub use crate::client::RqliteClientBuilder;
    pub use crate::query_result::QueryResult;
    pub use crate::FromRow;
    pub use crate::IntoTypedRows;
    pub use crate::Row;
}
