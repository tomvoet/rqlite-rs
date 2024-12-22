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
        // When fast-blob is disabled, we need to retrieve blob_arrays if possible because we dont decode base64
        #[cfg(not(feature = "fast-blob"))]
        let default_query_params = {
            let mut query_params = RqliteQueryParams::new()
                .blob_array()
                .into_request_query_params();

            if let Some(default_query_params) = self.default_query_params {
                let default_query_params = default_query_params.into_request_query_params();
                query_params.merge(default_query_params);
            }

            Some(query_params)
        };

        // When fast-blob is enabled, we dont need to retrieve blob_arrays because we decode base64
        #[cfg(feature = "fast-blob")]
        let default_query_params = self.default_query_params.map(RequestQueryParams::from);

        RqliteClientConfig {
            default_query_params,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_rqlite_client_config_builder() {
        let builder = RqliteClientConfigBuilder::default()
            .default_query_params(vec![RqliteQueryParam::Ver("3".to_string())])
            .scheme(Scheme::Https);

        let config = builder.build();

        assert!(config.default_query_params.is_some());
        assert!(matches!(config.scheme, Scheme::Https));
    }

    #[test]
    fn unit_rqlite_client_config() {
        let config = RqliteClientConfig::default();

        assert!(config.default_query_params.is_none());
        assert!(matches!(config.scheme, Scheme::Http));
    }

    #[test]
    fn unit_scheme() {
        assert_eq!(Scheme::Http.to_string(), "http");
        assert_eq!(Scheme::Https.to_string(), "https");
    }
}
