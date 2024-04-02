use std::{collections::HashMap, sync::Arc};

use serde_json::Value;

use crate::{column::Column, from_row::FromRow, IntoTypedError};

#[derive(Debug)]
pub struct Row {
    values: Box<[Value]>,
    columns: Arc<Vec<Column>>,
    column_names: Arc<HashMap<String, usize>>,
}

impl Row {
    pub fn new(
        columns: &Arc<Vec<Column>>,
        column_names: &Arc<HashMap<String, usize>>,
        values: Box<[Value]>,
    ) -> Row {
        Row {
            values,
            columns: Arc::clone(columns),
            column_names: Arc::clone(column_names),
        }
    }

    pub fn columns(&self) -> &Arc<Vec<Column>> {
        &self.columns
    }

    pub fn column_names(&self) -> &Arc<HashMap<String, usize>> {
        &self.column_names
    }

    pub fn get<T: serde::de::DeserializeOwned>(&self, name: &str) -> Result<T, IntoTypedError> {
        let index = self
            .column_names
            .get(name)
            .ok_or(IntoTypedError::ColumnNotFound)?;

        let value = self
            .values
            .get(*index)
            .ok_or(IntoTypedError::ValueNotFound)?;

        let value =
            serde_json::from_value(value.clone()).map_err(IntoTypedError::ConversionError)?;

        Ok(value)
    }

    pub fn get_opt<T: serde::de::DeserializeOwned>(
        &self,
        name: &str,
    ) -> Result<Option<T>, IntoTypedError> {
        let index = self
            .column_names
            .get(name)
            .ok_or(IntoTypedError::ColumnNotFound)?;

        let value = self
            .values
            .get(*index)
            .ok_or(IntoTypedError::ValueNotFound)?;

        match value {
            Value::Null => Ok(None),
            _ => {
                let value = serde_json::from_value(value.clone())
                    .map_err(IntoTypedError::ConversionError)?;
                Ok(Some(value))
            }
        }
    }

    pub fn get_by_index<T: serde::de::DeserializeOwned>(
        &self,
        index: usize,
    ) -> Result<T, IntoTypedError> {
        let value = self
            .values
            .get(index)
            .ok_or(IntoTypedError::ValueNotFound)?;
        let value =
            serde_json::from_value(value.clone()).map_err(IntoTypedError::ConversionError)?;
        Ok(value)
    }

    pub fn values(&self) -> &Box<[Value]> {
        &self.values
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn into_typed<T>(self) -> Result<T, IntoTypedError>
    where
        T: FromRow,
    {
        T::from_row(self)
    }
}
