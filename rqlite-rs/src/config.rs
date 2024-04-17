use crate::request::{RequestQueryParams, RqliteQueryParam, RqliteQueryParams};

#[derive(Default)]
pub(crate) struct RqliteClientConfigBuilder {
    pub(crate) default_query_params: Option<RqliteQueryParams>,
    pub(crate) scheme: Option<Scheme>,
}

impl RqliteClientConfigBuilder {
    pub(crate) fn default_query_params(mut self, params: Vec<RqliteQueryParam>) -> Self {
        self.default_query_params = Some(RqliteQueryParams::from(params));
        self
    }

    pub(crate) fn build(self) -> RqliteClientConfig {
        RqliteClientConfig {
            default_query_params: self.default_query_params.map(RequestQueryParams::from),
            scheme: self.scheme.unwrap_or(Scheme::Http),
        }
    }

    pub(crate) fn scheme(mut self, scheme: Scheme) -> Self {
        self.scheme = Some(scheme);
        self
    }
}

#[derive(Default)]
pub(crate) struct RqliteClientConfig {
    pub(crate) default_query_params: Option<RequestQueryParams>,
    pub(crate) scheme: Scheme,
}

#[derive(Default)]
pub enum Scheme {
    #[default]
    Http,
    Https,
}

impl std::fmt::Display for Scheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Scheme::Http => write!(f, "http"),
            Scheme::Https => write!(f, "https"),
        }
    }
}
