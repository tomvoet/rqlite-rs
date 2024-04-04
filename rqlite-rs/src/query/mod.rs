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

#[derive(Serialize, Debug)]
pub(crate) struct QueryArgs(Vec<Vec<QueryComponent>>);

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
            Operation::Create => "execute",
            Operation::Select => "query",
            Operation::Update => "execute",
            Operation::Delete => "execute",
            Operation::Insert => "execute",
            Operation::Pragma => "query",
            Operation::Drop => "execute",
        };

        format!("db/{}", resource)
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
#[macro_export]
macro_rules! query {
    // This is the base case, it only accepts a query string.
    // In this macro named blocks are used to allow using early returns.
    ( $query:expr ) => {{
        'blk: {
            let op = match $crate::query::Operation::from_query_string($query) {
                Ok(op) => op,
                Err(_) => break 'blk Err($crate::error::QueryBuilderError::InvalidQuery($query.to_string())),
            };

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
    #[test]
    fn test_query_macro() {
        let query = query!("SELECT * FROM foo");
        assert!(query.is_ok());
        assert_eq!(query.unwrap().args.len(), 0);

        let query = query!("SELECT * FROM foo WHERE id = ?", 1i64);
        assert!(query.is_ok());
        assert_eq!(query.unwrap().args.len(), 1);

        let query = query!("SELECT * FROM foo WHERE id = ? AND name = ?", 1i64, "foo");
        assert!(query.is_ok());
        assert_eq!(query.unwrap().args.len(), 2);

        let query = query!("SELECT * FROM foo WHERE id = ? AND name = ?", 1i64);
        assert!(query.is_err());
    }
}
