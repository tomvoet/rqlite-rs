use std::{collections::HashMap, sync::Arc};

use rqlite_rs_core::{Column, Row};
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize, Clone)]
pub struct RqliteSelectResults {
    columns: Vec<String>,
    types: Vec<String>,
    values: Option<Vec<Vec<Value>>>,
}

impl RqliteSelectResults {
    pub fn rows(&self) -> anyhow::Result<Vec<Row>> {
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
