use serde::Deserialize;

/// The result of a query.
#[derive(Debug, Deserialize, Clone)]
pub struct QueryResult {
    /// The last insert ID.
    last_insert_id: i64,
    /// The number of rows affected.
    rows_affected: i64,
}

impl QueryResult {
    /// Returns the last insert ID, if any.
    pub fn last_insert_id(&self) -> i64 {
        self.last_insert_id
    }

    /// Returns the number of rows affected, if any.
    pub fn rows_affected(&self) -> i64 {
        self.rows_affected
    }

    /// Returns `true` if the query changed the database.
    pub fn changed(&self) -> bool {
        self.rows_affected > 0
    }
}
