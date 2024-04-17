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
pub(crate) mod select;

#[cfg(feature = "macros")]
pub use rqlite_rs_macros::*;

pub mod prelude {
    pub use crate::client::RqliteClient;
    pub use crate::client::RqliteClientBuilder;
    pub use crate::query_result::QueryResult;
    pub use crate::FromRow;
    pub use crate::IntoTypedRows;
    pub use crate::Row;
}
