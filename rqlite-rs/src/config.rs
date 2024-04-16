use crate::request::{RequestQueryParams, RqliteQueryParam, RqliteQueryParams};

#[derive(Default)]
pub(crate) struct RqliteClientConfigBuilder {
    pub(crate) default_query_params: Option<RqliteQueryParams>,
}

impl RqliteClientConfigBuilder {
    pub(crate) fn default_query_params(mut self, params: Vec<RqliteQueryParam>) -> Self {
        self.default_query_params = Some(RqliteQueryParams::from(params));
        self
    }

    pub(crate) fn build(self) -> RqliteClientConfig {
        RqliteClientConfig {
            default_query_params: self.default_query_params.map(RequestQueryParams::from),
        }
    }
}

#[derive(Default)]
pub(crate) struct RqliteClientConfig {
    pub(crate) default_query_params: Option<RequestQueryParams>,
}
