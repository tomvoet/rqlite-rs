use serde::Serialize;

#[derive(Debug, PartialEq, Serialize, Clone)]
#[serde(untagged)]
pub enum RqLiteArgument {
    String(String),
    I64(i64),
    F64(f64),
    F32(f32),
    Bool(bool),
}

impl RqLiteArgument {}

pub trait RqLiteArgumentRaw {
    fn encode(&self) -> RqLiteArgument;
}

impl RqLiteArgumentRaw for i64 {
    fn encode(&self) -> RqLiteArgument {
        RqLiteArgument::I64(self.to_owned())
    }
}

impl RqLiteArgumentRaw for usize {
    fn encode(&self) -> RqLiteArgument {
        RqLiteArgument::I64(*self as i64)
    }
}

impl RqLiteArgumentRaw for f64 {
    fn encode(&self) -> RqLiteArgument {
        RqLiteArgument::F64(self.to_owned())
    }
}

impl RqLiteArgumentRaw for f32 {
    fn encode(&self) -> RqLiteArgument {
        RqLiteArgument::F32(self.to_owned())
    }
}

impl RqLiteArgumentRaw for bool {
    fn encode(&self) -> RqLiteArgument {
        RqLiteArgument::Bool(self.to_owned())
    }
}

impl RqLiteArgumentRaw for &str {
    fn encode(&self) -> RqLiteArgument {
        RqLiteArgument::String(self.to_string())
    }
}

impl RqLiteArgumentRaw for String {
    fn encode(&self) -> RqLiteArgument {
        RqLiteArgument::String(self.to_string())
    }
}

#[macro_export]
macro_rules! arg {
    ($e:expr) => {
        $crate::query::arguments::RqLiteArgumentRaw::encode(&$e)
    };
}
