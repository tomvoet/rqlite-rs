use std::str::FromStr;

use reqwest::Url;
use serde::Serialize;
use serde_json;

pub mod arguments;
pub(crate) use arguments::RqliteArgument;

#[derive(Debug)]
pub struct RqliteQuery {
    pub query: String,
    pub args: Vec<RqliteArgument>,
    pub op: Operation,
}

#[derive(Serialize, Debug)]
struct QueryComponent(RqliteArgument);

#[derive(Serialize, Debug)]
pub(crate) struct QueryArgs(Vec<Vec<QueryComponent>>);

impl QueryArgs {
    fn new(query: String) -> QueryArgs {
        QueryArgs(vec![vec![QueryComponent(RqliteArgument::String(query))]])
    }

    fn add(&mut self, arg: RqliteArgument) {
        self.0[0].push(QueryComponent(arg));
    }
}

impl RqliteQuery {
    pub(crate) fn to_json(&self) -> anyhow::Result<String> {
        let query = self.query.clone();
        let mut args = QueryArgs::new(query);

        for arg in &self.args {
            args.add(arg.clone());
        }

        Ok(serde_json::to_string(&args)?)
    }

    pub(crate) fn url(&self, host: &str) -> anyhow::Result<Url> {
        let endpoint = match self.op {
            Operation::Create => "execute",
            Operation::Select => "query",
            Operation::Update => "execute",
            Operation::Delete => "execute",
            Operation::Insert => "execute",
            Operation::Pragma => "query",
        };

        let url = format!("http://{}/db/{}", host, endpoint);

        Ok(Url::from_str(&url)?)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Operation {
    Create,
    Select,
    Update,
    Delete,
    Insert,
    Pragma,
}

#[macro_export]
macro_rules! query {
    ( $query:expr ) => {{
        let lower = $query.to_lowercase();

        let op = if lower.starts_with("create") {
            $crate::query::Operation::Create
        } else if lower.starts_with("select") {
            $crate::query::Operation::Select
        } else if lower.starts_with("update") {
            $crate::query::Operation::Update
        } else if lower.starts_with("delete") {
            $crate::query::Operation::Delete
        } else if lower.starts_with("insert") {
            $crate::query::Operation::Insert
        } else if lower.starts_with("pragma") {
            $crate::query::Operation::Pragma
        } else {
            anyhow::bail!("Invalid query");
        };

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
