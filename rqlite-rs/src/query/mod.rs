use serde::Serialize;
use serde_json;

pub mod arguments;
use crate::error::QueryBuilderError;
pub(crate) use arguments::RqliteArgument;

/// A query to be executed on the rqlite cluster.
#[derive(Debug)]
pub struct RqliteQuery {
    pub query: String,
    pub args: Vec<RqliteArgument>,
    pub op: Operation,
}

impl TryInto<RqliteQuery> for String {
    type Error = QueryBuilderError;

    /// Attempts to convert a string into a [`RqliteQuery`].
    /// Returns a `Result` with the query if the string is valid.
    /// Fails if the query does not start with a valid operation.
    /// See [`Operation`] for a list of valid operations.
    fn try_into(self) -> Result<RqliteQuery, Self::Error> {
        let op = Operation::from_query_string(self.as_str())?;

        Ok(RqliteQuery {
            query: self,
            args: vec![],
            op,
        })
    }
}

impl TryInto<RqliteQuery> for &str {
    type Error = QueryBuilderError;

    /// Attempts to convert a string into a [`RqliteQuery`].
    /// Returns a `Result` with the query if the string is valid.
    /// Fails if the query does not start with a valid operation.
    /// See [`Operation`] for a list of valid operations.
    fn try_into(self) -> Result<RqliteQuery, Self::Error> {
        let op = Operation::from_query_string(self)?;

        Ok(RqliteQuery {
            query: self.to_string(),
            args: vec![],
            op,
        })
    }
}

impl TryInto<RqliteQuery> for Result<RqliteQuery, QueryBuilderError> {
    type Error = QueryBuilderError;

    /// Attempts to convert a `Result` into a [`RqliteQuery`].
    /// Returns a `Result` with the query if the result is valid.
    /// Fails if the result is an error.
    /// This allows avoiding the use of `?` in the `query!` macro.
    fn try_into(self) -> Result<RqliteQuery, Self::Error> {
        self
    }
}

#[derive(Serialize, Debug)]
struct QueryComponent(RqliteArgument);

// This is a helper struct for serializing multiple queries with arguments.
#[derive(Serialize, Debug)]
pub(crate) struct QueryArgs(Vec<Vec<QueryComponent>>);

// This is an internal helper, for creating a `QueryArgs` from a vector of `RqliteQuery`.
// This is used for batch queries.
impl From<RqliteQuery> for QueryArgs {
    fn from(query: RqliteQuery) -> Self {
        let mut args = Vec::new();

        let mut components = Vec::new();

        components.push(QueryComponent(RqliteArgument::String(query.query)));

        for arg in query.args {
            components.push(QueryComponent(arg));
        }

        args.push(components);

        QueryArgs(args)
    }
}

// This is an internal helper, for creating a `QueryArgs` from a vector of `RqliteQuery`.
// This is used for batch queries.
impl From<Vec<RqliteQuery>> for QueryArgs {
    fn from(queries: Vec<RqliteQuery>) -> Self {
        let mut args = Vec::new();

        for query in queries {
            let mut components = Vec::new();

            components.push(QueryComponent(RqliteArgument::String(query.query)));

            for arg in query.args {
                components.push(QueryComponent(arg));
            }

            args.push(components);
        }

        QueryArgs(args)
    }
}

impl RqliteQuery {
    pub(crate) fn into_json(self) -> Result<String, serde_json::Error> {
        let args = QueryArgs::from(self);

        serde_json::to_string(&args)
    }

    pub(crate) fn endpoint(&self) -> String {
        let resource = match self.op {
            Operation::Create
            | Operation::Update
            | Operation::Delete
            | Operation::Insert
            | Operation::Drop => "execute",
            Operation::Select | Operation::Pragma => "query",
        };

        format!("db/{resource}")
    }
}

/// The type of operation for a query.
#[derive(Debug, PartialEq, Eq)]
pub enum Operation {
    Create,
    Select,
    Update,
    Delete,
    Insert,
    Pragma,
    Drop,
}

impl Operation {
    /// Convert a SQL query string into an [`Operation`].
    ///
    /// # Errors
    /// Returns [`QueryBuilderError::InvalidOperation`] if the query string does not start with a valid operation keyword.
    pub fn from_query_string(query: &str) -> Result<Operation, QueryBuilderError> {
        match query.to_lowercase() {
            q if q.starts_with("create") => Ok(Operation::Create),
            q if q.starts_with("select") => Ok(Operation::Select),
            q if q.starts_with("update") => Ok(Operation::Update),
            q if q.starts_with("delete") => Ok(Operation::Delete),
            q if q.starts_with("insert") => Ok(Operation::Insert),
            q if q.starts_with("pragma") => Ok(Operation::Pragma),
            q if q.starts_with("drop") => Ok(Operation::Drop),
            _ => Err(QueryBuilderError::InvalidOperation(query.to_string())),
        }
    }
}

/// A macro for creating a query.
/// Returns a `Result` with an [`RqliteQuery`] if the query is valid.
/// The macro accepts a query string and optional arguments.
///
/// # Examples
///
/// ```
/// use rqlite_rs::query;
///
/// let query = query!("SELECT * FROM foo");
/// assert!(query.is_ok());
///
/// let query = query!("SELECT * FROM foo WHERE id = ? AND name = ?", 1i64, "bar");
/// assert!(query.is_ok());
/// ```
#[macro_export]
macro_rules! query {
    // This is the base case, it only accepts a query string.
    // In this macro named blocks are used to allow using early returns.
    ( $query:expr ) => {{
        'blk: {
            let Ok(op) = $crate::query::Operation::from_query_string($query) else { break 'blk Err($crate::error::QueryBuilderError::InvalidQuery($query.to_string())) };

            Ok($crate::query::RqliteQuery {
                query: $query.to_string(),
                args: vec![],
                op,
            })
        }
    }};
    ( $query:expr, $( $args:expr ),* ) => {{
        'blk: {
            let Ok(query) = ($crate::query!($query)) else {
                // This is not unreachable.
                #[allow(unreachable_code)]
                break 'blk Err($crate::error::QueryBuilderError::InvalidQuery($query.to_string()));
            };

            let param_count = $query.matches("?").count();

            let mut args = vec![];

            $(
                let arg = $crate::arg!($args);
                args.push(arg);
            )*

            let argc = args.len();

            if argc != param_count {
                break 'blk Err($crate::error::QueryBuilderError::InvalidArgumentCount(param_count, argc));
            }

            Ok($crate::query::RqliteQuery {
                query: query.query,
                args,
                op: query.op,
            })
        }
    }};
}

#[cfg(test)]
mod tests {
    use crate::query::QueryArgs;

    // This is a unit test for the query macro.
    #[test]
    fn unit_query_macro_correct_query_types() {
        let query = query!("CREATE TABLE foo (id INTEGER PRIMARY KEY)");
        assert!(query.is_ok());
        assert_eq!(query.unwrap().op, crate::query::Operation::Create);

        let query = query!("SELECT * FROM foo");
        assert!(query.is_ok());
        assert_eq!(query.unwrap().op, crate::query::Operation::Select);

        let query = query!("UPDATE foo SET name = 'bar' WHERE id = 1");
        assert!(query.is_ok());
        assert_eq!(query.unwrap().op, crate::query::Operation::Update);

        let query = query!("DELETE FROM foo WHERE id = 1");
        assert!(query.is_ok());
        assert_eq!(query.unwrap().op, crate::query::Operation::Delete);

        let query = query!("INSERT INTO foo (name) VALUES ('bar')");
        assert!(query.is_ok());
        assert_eq!(query.unwrap().op, crate::query::Operation::Insert);

        let query = query!("PRAGMA table_info(foo)");
        assert!(query.is_ok());
        assert_eq!(query.unwrap().op, crate::query::Operation::Pragma);

        let query = query!("DROP TABLE foo");
        assert!(query.is_ok());
        assert_eq!(query.unwrap().op, crate::query::Operation::Drop);
    }

    #[test]
    fn unit_query_macro_incorrect_query_types() {
        let query = query!("TEST * FROM foo");
        assert!(query.is_err());
    }

    #[test]
    fn unit_query_macro_query_args() {
        let query = query!("SELECT * FROM foo WHERE id = ?", 1i64);
        assert!(query.is_ok());
        assert_eq!(query.unwrap().args.len(), 1);

        let query = query!("SELECT * FROM foo WHERE id = ?", 1i64, "foo");
        assert!(query.is_err());
        assert!(matches!(
            query.unwrap_err(),
            crate::error::QueryBuilderError::InvalidArgumentCount(1, 2)
        ));
    }

    #[test]
    fn unit_query_macro_try_into_from_string_slice() {
        let query: Result<crate::query::RqliteQuery, crate::error::QueryBuilderError> =
            "SELECT * FROM foo".try_into();
        assert!(query.is_ok());
        assert_eq!(query.unwrap().op, crate::query::Operation::Select);
    }

    #[test]
    fn unit_query_macro_try_into_from_string() {
        let query: Result<crate::query::RqliteQuery, crate::error::QueryBuilderError> =
            "SELECT * FROM foo".to_string().try_into();
        assert!(query.is_ok());
        assert_eq!(query.unwrap().op, crate::query::Operation::Select);
    }

    #[test]
    fn unit_query_macro_try_into_from_result_err() {
        let query: Result<crate::query::RqliteQuery, crate::error::QueryBuilderError> = Err(
            crate::error::QueryBuilderError::InvalidQuery("TEST * FROM foo".to_string()),
        )
        .try_into();
        assert!(query.is_err());
    }

    #[test]
    fn unit_query_macro_query_args_from_query() {
        let query = query!("SELECT * FROM foo WHERE id = ?", 1i64);
        assert!(query.is_ok());

        let query_args = QueryArgs::from(query.unwrap());
        assert_eq!(query_args.0.len(), 1);
        assert_eq!(query_args.0[0].len(), 2);
    }

    #[test]
    fn unit_query_macro_query_args_from_query_vec() {
        let query_1 = query!("SELECT * FROM foo WHERE id = ?", 1i64);
        let query_2 = query!("SELECT * FROM foo WHERE id = ?", 2i64);
        assert!(query_1.is_ok());
        assert!(query_2.is_ok());

        let query_args = QueryArgs::from(vec![query_1.unwrap(), query_2.unwrap()]);
        assert_eq!(query_args.0.len(), 2);
        assert_eq!(query_args.0[0].len(), 2);
        assert_eq!(query_args.0[1].len(), 2);
    }

    #[test]
    fn unit_query_macro_query_to_json() {
        let query = query!("SELECT * FROM foo WHERE id = ?", 1i64);
        assert!(query.is_ok());

        let json = query.unwrap().into_json();
        assert_eq!(json.unwrap(), r#"[["SELECT * FROM foo WHERE id = ?",1]]"#);
    }

    #[test]
    fn unit_query_macro_query_endpoint() {
        let query = query!("SELECT * FROM foo WHERE id = ?", 1i64);
        assert!(query.is_ok());

        let endpoint = query.unwrap().endpoint();
        assert_eq!(endpoint, "db/query");
    }
}
