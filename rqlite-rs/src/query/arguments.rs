use serde::Serialize;

#[derive(Debug, PartialEq, Serialize, Clone)]
#[serde(untagged)]
pub enum RqliteArgument {
    String(String),
    I64(i64),
    F64(f64),
    F32(f32),
    Bool(bool),
}

impl RqliteArgument {}

pub trait RqliteArgumentRaw {
    fn encode(&self) -> RqliteArgument;
}

impl RqliteArgumentRaw for i64 {
    fn encode(&self) -> RqliteArgument {
        RqliteArgument::I64(self.to_owned())
    }
}

impl RqliteArgumentRaw for usize {
    fn encode(&self) -> RqliteArgument {
        RqliteArgument::I64(*self as i64)
    }
}

impl RqliteArgumentRaw for f64 {
    fn encode(&self) -> RqliteArgument {
        RqliteArgument::F64(self.to_owned())
    }
}

impl RqliteArgumentRaw for f32 {
    fn encode(&self) -> RqliteArgument {
        RqliteArgument::F32(self.to_owned())
    }
}

impl RqliteArgumentRaw for bool {
    fn encode(&self) -> RqliteArgument {
        RqliteArgument::Bool(self.to_owned())
    }
}

impl RqliteArgumentRaw for &str {
    fn encode(&self) -> RqliteArgument {
        RqliteArgument::String(self.to_string())
    }
}

impl RqliteArgumentRaw for String {
    fn encode(&self) -> RqliteArgument {
        RqliteArgument::String(self.to_string())
    }
}

#[macro_export]
macro_rules! arg {
    ($e:expr) => {
        $crate::query::arguments::RqliteArgumentRaw::encode(&$e)
    };
}
