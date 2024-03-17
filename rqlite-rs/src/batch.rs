use crate::{prelude::QueryResult, select::RqliteSelectResults};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum BatchResult {
    SelectResults(RqliteSelectResults),
    QueryResult(QueryResult),
}
