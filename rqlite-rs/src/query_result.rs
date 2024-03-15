use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub(crate) enum QueryResultRaw<T> {
    Success(T),
    Error(QueryError),
}

/// The result of a query.
#[derive(Debug, Deserialize, Clone)]
pub struct QueryResult {
    /// The last insert ID.
    last_insert_id: Option<i64>,
    /// The number of rows affected.
    rows_affected: Option<i64>,
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct QueryError {
    pub(crate) error: String,
}

impl QueryResult {
    /// Returns the last insert ID, if any.
    pub fn last_insert_id(&self) -> Option<i64> {
        self.last_insert_id
    }

    /// Returns the number of rows affected, if any.
    pub fn rows_affected(&self) -> Option<i64> {
        self.rows_affected
    }

    /// Returns `true` if the query changed the database.
    pub fn changed(&self) -> bool {
        self.rows_affected.unwrap_or(0) > 0
    }
}
