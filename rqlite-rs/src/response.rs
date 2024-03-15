use std::{collections::HashMap, sync::Arc};

use serde::Deserialize;
use serde_json::Value;

use crate::query_result::QueryResultRaw;
use rqlite_rs_core::{Column, Row};

#[derive(Debug, Deserialize)]
pub(crate) struct RqliteResponseRaw<T> {
    pub(crate) results: Vec<QueryResultRaw<T>>,
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct RqliteSelectResponseRawResults {
    columns: Vec<String>,
    types: Vec<String>,
    values: Option<Vec<Vec<Value>>>,
}

impl RqliteSelectResponseRawResults {
    pub(crate) fn rows(&self) -> anyhow::Result<Vec<Row>> {
        let mut rows = Vec::new();

        let mut columns = vec![];
        let mut column_names = HashMap::new();

        for (index, column) in self.columns.iter().enumerate() {
            let column = Column::new(
                column.to_string(),
                index,
                self.types.get(index).unwrap().clone(),
            );

            column_names.insert(column.name().to_string(), index);
            columns.push(column);
        }

        let columns = Arc::new(columns);
        let column_names = Arc::new(column_names);

        if let Some(values) = &self.values {
            for row in values {
                rows.push(Row::new(
                    &columns,
                    &column_names,
                    row.clone().into_boxed_slice(), //TODO! unnecessary clone
                ));
            }
        }

        Ok(rows)
    }
}
