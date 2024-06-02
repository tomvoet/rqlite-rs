#[derive(Debug)]
pub struct Column {
    name: String,
    ordinal: usize,
    type_data: String,
}

impl Column {
    pub fn new(name: String, ordinal: usize, type_data: String) -> Column {
        Column {
            name,
            ordinal,
            type_data,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn ordinal(&self) -> usize {
        self.ordinal
    }

    pub fn type_data(&self) -> &str {
        &self.type_data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_column() {
        let column = Column::new("name".to_string(), 0, "TEXT".to_string());
        assert_eq!(column.name(), "name");
        assert_eq!(column.ordinal(), 0);
        assert_eq!(column.type_data(), "TEXT");
    }
}
