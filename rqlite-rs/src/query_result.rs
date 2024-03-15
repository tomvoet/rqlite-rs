use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub(crate) enum QueryResultRaw<T> {
    Success(T),
    Error(QueryError),
}

#[derive(Debug, Deserialize, Clone)]
pub struct QueryResult {
    last_insert_id: Option<i64>,
    rows_affected: Option<i64>,
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct QueryError {
    pub(crate) error: String,
}

impl QueryResult {
    pub fn last_insert_id(&self) -> Option<i64> {
        self.last_insert_id
    }

    pub fn rows_affected(&self) -> Option<i64> {
        self.rows_affected
    }

    pub fn changed(&self) -> bool {
        self.rows_affected.unwrap_or(0) > 0
    }
}
