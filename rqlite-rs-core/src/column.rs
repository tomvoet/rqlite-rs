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
