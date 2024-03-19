use std::any;

use serde::Serialize;
use serde_json;

pub mod arguments;
pub(crate) use arguments::RqliteArgument;

/// A query to be executed on the rqlite cluster.
#[derive(Debug)]
pub struct RqliteQuery {
    pub query: String,
    pub args: Vec<RqliteArgument>,
    pub op: Operation,
}

impl TryInto<RqliteQuery> for String {
    type Error = anyhow::Error;

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
    type Error = anyhow::Error;

    /// Attempts to convert a string into a [`RqliteQuery`].
    /// Returns a `Result` with the query if the string is valid.
    /// Fails if the query does not start with a valid operation.
    /// See [`Operation`] for a list of valid operations.
    fn try_into(self) -> Result<RqliteQuery, Self::Error> {
        let op = Operation::from_query_string(&self)?;

        Ok(RqliteQuery {
            query: self.to_string(),
            args: vec![],
            op,
        })
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
    pub(crate) fn to_json(self) -> anyhow::Result<String> {
        let args = QueryArgs::from(self);

        Ok(serde_json::to_string(&args)?)
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
    pub fn from_query_string(query: &str) -> anyhow::Result<Self> {
        match query.to_lowercase() {
            q if q.starts_with("create") => Ok(Operation::Create),
            q if q.starts_with("select") => Ok(Operation::Select),
            q if q.starts_with("update") => Ok(Operation::Update),
            q if q.starts_with("delete") => Ok(Operation::Delete),
            q if q.starts_with("insert") => Ok(Operation::Insert),
            q if q.starts_with("pragma") => Ok(Operation::Pragma),
            q if q.starts_with("drop") => Ok(Operation::Drop),
            _ => anyhow::bail!("Invalid query"),
        }
    }
}

/// A macro for creating a query.
/// Returns a `Result` with an [`RqliteQuery`] if the query is valid.
/// # Examples
/// ```
/// # use rqlite_rs::prelude::*;
/// let query = query!("SELECT * FROM foo");
/// assert!(query.is_ok());
///
/// let query = query!("SELECT * FROM foo WHERE id = ?", 1);
/// assert!(query.is_ok());
/// assert_eq!(query.unwrap().args.len(), 1);
/// ```
#[macro_export]
macro_rules! query {
    ( $query:expr ) => {{
        let lower = $query.to_lowercase();

        //let op = if lower.starts_with("create") {
        //    $crate::query::Operation::Create
        //} else if lower.starts_with("select") {
        //    $crate::query::Operation::Select
        //} else if lower.starts_with("update") {
        //    $crate::query::Operation::Update
        //} else if lower.starts_with("delete") {
        //    $crate::query::Operation::Delete
        //} else if lower.starts_with("insert") {
        //    $crate::query::Operation::Insert
        //} else if lower.starts_with("pragma") {
        //    $crate::query::Operation::Pragma
        //} else if lower.starts_with("drop") {
        //    $crate::query::Operation::Drop
        //} else {
        //    anyhow::bail!("Invalid query");
        //};
        let op = $crate::query::Operation::from_query_string($query)?;

        Ok($crate::query::RqliteQuery {
            query: $query.to_string(),
            args: vec![],
            op,
        }) as anyhow::Result<$crate::query::RqliteQuery>
    }};
    ( $query:expr, $( $args:expr ),* ) => {{
        let Ok(query) = $crate::query!($query) else {
            // This is not unreachable.
            #[allow(unreachable_code)]
            return anyhow::bail!("Invalid query");
        };

        let param_count = $query.matches("?").count();

        let mut args = vec![];

        $(
            let arg = $crate::arg!($args);
            args.push(arg);
        )*

        let argc = args.len();

        if argc != param_count {
            anyhow::bail!("Invalid number of arguments, expected {}, got {}", param_count, argc);
        }

        Ok($crate::query::RqliteQuery {
            query: query.query,
            args,
            op: query.op,
        }) as anyhow::Result<$crate::query::RqliteQuery>
    }};
}
