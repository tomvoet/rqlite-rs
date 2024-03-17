use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum RqliteResult<T> {
    Success(T),
    Error(QueryError),
}

#[derive(Debug, Deserialize, Clone)]
pub struct QueryError {
    pub error: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RqliteResponseRaw<T> {
    pub(crate) results: Vec<RqliteResult<T>>,
}
