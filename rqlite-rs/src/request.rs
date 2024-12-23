use std::{fmt::Display, num::NonZeroU16};

use serde::Serialize;

use crate::config::Scheme;

pub(crate) struct RequestOptions {
    pub(crate) method: reqwest::Method,
    pub(crate) endpoint: String,
    pub(crate) body: Option<String>,
    pub(crate) params: Option<RequestQueryParams>,
}

impl Default for RequestOptions {
    fn default() -> Self {
        RequestOptions {
            method: reqwest::Method::POST,
            endpoint: "db/request".to_string(),
            body: None,
            params: None,
        }
    }
}

impl RequestOptions {
    pub(crate) fn to_reqwest_request(
        &self,
        client: &reqwest::Client,
        host: &str,
        scheme: &Scheme,
    ) -> reqwest::RequestBuilder {
        let mut req = client.request(
            self.method.clone(),
            format!("{}://{}/{}", scheme, host, self.endpoint),
        );

        if let Some(body) = &self.body {
            req = req.body(body.clone());
        }

        if let Some(params) = &self.params {
            req = req.query(&params.clone().into_reqwest_query());
        }

        req
    }

    pub(crate) fn merge_default_query_params(&mut self, default_params: &RequestQueryParams) {
        if let Some(params) = &mut self.params {
            params.merge(default_params.clone());
        } else {
            self.params = Some(default_params.clone());
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub(crate) enum RequestQueryParam {
    Bool(String),
    KV(String, String),
}

impl RequestQueryParam {
    fn into_reqwest_query(self) -> (String, String) {
        match self {
            RequestQueryParam::Bool(k) => (k, "true".to_string()),
            RequestQueryParam::KV(k, v) => (k, v),
        }
    }

    fn is_same_key(&self, other: &RequestQueryParam) -> bool {
        match (self, other) {
            (
                RequestQueryParam::Bool(k1) | RequestQueryParam::KV(k1, _),
                RequestQueryParam::Bool(k2) | RequestQueryParam::KV(k2, _),
            ) => k1 == k2,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct RequestQueryParams(pub(super) Vec<RequestQueryParam>);

impl RequestQueryParams {
    pub(crate) fn new() -> Self {
        RequestQueryParams::default()
    }

    pub(crate) fn into_reqwest_query(self) -> Vec<(String, String)> {
        self.0
            .into_iter()
            .map(RequestQueryParam::into_reqwest_query)
            .collect()
    }

    pub(crate) fn merge(&mut self, other: RequestQueryParams) {
        //self.0.extend(other.0.clone());#
        //Deduplication
        for p in other.0 {
            if !self.0.iter().any(|x| x.is_same_key(&p)) {
                self.0.push(p);
            }
        }
    }
}

#[allow(dead_code)]
pub enum RqliteFreshnessLevel {
    None,
    Weak,
    Strong,
}

impl Display for RqliteFreshnessLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RqliteFreshnessLevel::None => write!(f, "none"),
            RqliteFreshnessLevel::Weak => write!(f, "weak"),
            RqliteFreshnessLevel::Strong => write!(f, "strong"),
        }
    }
}

pub enum RqliteQueryParam {
    /// Pretty print the response (not necessary, since we're parsing it anyway)
    Pretty,
    /// Include timing information in the response
    Timings,
    /// Execute the query in a transaction
    Transaction,
    /// Queue the query for later execution
    Queue,
    /// Timeout in seconds
    Timeout(NonZeroU16),
    /// Freshness level
    Level(RqliteFreshnessLevel),
    /// Freshness in seconds
    Freshness(NonZeroU16),
    /// Strict freshness
    FreshnessStrict,
    /// Disable `RANDOM()` rewriting
    NoRWRandom,
    /// Version
    Ver(String),
    /// Retrieve blobs as u8 arrays instead of base64 encoded strings
    /// Defaults to true when the `fast-blob` feature is disabled
    BlobArray,
}

impl RqliteQueryParam {
    fn into_request_query_param(self) -> RequestQueryParam {
        match self {
            RqliteQueryParam::Pretty => RequestQueryParam::Bool("pretty".to_string()),
            RqliteQueryParam::Timings => RequestQueryParam::Bool("timings".to_string()),
            RqliteQueryParam::Transaction => RequestQueryParam::Bool("transaction".to_string()),
            RqliteQueryParam::Queue => RequestQueryParam::Bool("queue".to_string()),
            RqliteQueryParam::Timeout(t) => {
                RequestQueryParam::KV("timeout".to_string(), format!("{t}s"))
            }
            RqliteQueryParam::Level(l) => RequestQueryParam::KV("level".to_string(), l.to_string()),
            RqliteQueryParam::Freshness(f) => {
                RequestQueryParam::KV("freshness".to_string(), format!("{f}s"))
            }
            RqliteQueryParam::FreshnessStrict => {
                RequestQueryParam::Bool("freshness_strict".to_string())
            }
            RqliteQueryParam::NoRWRandom => RequestQueryParam::Bool("norwrandom".to_string()),
            RqliteQueryParam::Ver(v) => RequestQueryParam::KV("ver".to_string(), v),
            RqliteQueryParam::BlobArray => RequestQueryParam::Bool("blob_array".to_string()),
        }
    }
}

#[derive(Default)]
pub(crate) struct RqliteQueryParams(Vec<RqliteQueryParam>);

#[allow(dead_code)]
impl RqliteQueryParams {
    pub fn new() -> Self {
        RqliteQueryParams(Vec::new())
    }

    pub fn pretty(mut self) -> Self {
        self.0.push(RqliteQueryParam::Pretty);
        self
    }

    pub fn timings(mut self) -> Self {
        self.0.push(RqliteQueryParam::Timings);
        self
    }

    pub fn transaction(mut self) -> Self {
        self.0.push(RqliteQueryParam::Transaction);
        self
    }

    pub fn queue(mut self) -> Self {
        self.0.push(RqliteQueryParam::Queue);
        self
    }

    pub fn timeout(mut self, t: NonZeroU16) -> Self {
        self.0.push(RqliteQueryParam::Timeout(t));
        self
    }

    pub fn level(mut self, l: RqliteFreshnessLevel) -> Self {
        self.0.push(RqliteQueryParam::Level(l));
        self
    }

    pub fn freshness(mut self, f: NonZeroU16) -> Self {
        self.0.push(RqliteQueryParam::Freshness(f));
        self
    }

    pub fn freshness_strict(mut self) -> Self {
        self.0.push(RqliteQueryParam::FreshnessStrict);
        self
    }

    pub fn norwrandom(mut self) -> Self {
        self.0.push(RqliteQueryParam::NoRWRandom);
        self
    }

    pub fn ver(mut self, v: String) -> Self {
        self.0.push(RqliteQueryParam::Ver(v));
        self
    }

    pub fn blob_array(mut self) -> Self {
        self.0.push(RqliteQueryParam::BlobArray);
        self
    }

    pub(crate) fn into_request_query_params(self) -> RequestQueryParams {
        let mut params = RequestQueryParams::new();

        for p in self.0 {
            params.0.push(p.into_request_query_param());
        }

        params
    }
}

impl From<Vec<RqliteQueryParam>> for RqliteQueryParams {
    fn from(params: Vec<RqliteQueryParam>) -> Self {
        RqliteQueryParams(params)
    }
}

impl From<RqliteQueryParams> for RequestQueryParams {
    fn from(params: RqliteQueryParams) -> Self {
        params.into_request_query_params()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn full_query_params() -> RqliteQueryParams {
        RqliteQueryParams::new()
            .pretty()
            .timings()
            .transaction()
            .queue()
            .timeout(NonZeroU16::new(10).unwrap())
            .level(RqliteFreshnessLevel::Weak)
            .freshness(NonZeroU16::new(5).unwrap())
            .freshness_strict()
            .norwrandom()
            .ver("1".to_string())
            .blob_array()
    }

    #[test]
    fn unit_request_query_params() {
        let params = full_query_params();
        let req_params = RequestQueryParams::from(params);

        #[cfg(feature = "fast-blob")]
        assert_eq!(req_params.0.len(), 11);
        #[cfg(not(feature = "fast-blob"))]
        assert_eq!(req_params.0.len(), 12);
    }

    #[test]
    fn unit_request_query_params_into_reqwest_query() {
        let params = full_query_params();
        let req_params = RequestQueryParams::from(params);

        let reqwest_query = req_params.into_reqwest_query();

        #[cfg(feature = "fast-blob")]
        assert_eq!(reqwest_query.len(), 11);
        #[cfg(not(feature = "fast-blob"))]
        assert_eq!(reqwest_query.len(), 12);
    }

    #[test]
    fn unit_request_options_default() {
        let req = RequestOptions::default();

        assert_eq!(req.method, reqwest::Method::POST);
        assert_eq!(req.endpoint, "db/request");
        assert_eq!(req.body, None);
        assert!(req.params.is_none());
    }

    #[test]
    fn unit_request_options_to_reqwest_request() {
        let req = RequestOptions {
            params: Some(RequestQueryParams::from(full_query_params())),
            ..Default::default()
        };
        let client = reqwest::Client::new();
        let host = "localhost";
        let scheme = Scheme::Http;

        let reqwest_req = req.to_reqwest_request(&client, host, &scheme);

        let reqwest_req = reqwest_req.build().unwrap();

        assert_eq!(reqwest_req.method(), reqwest::Method::POST);

        // check query scheme, host and params
        let url = reqwest_req.url().to_string();

        assert!(url.starts_with("http://localhost/db/request"));

        let query = reqwest_req.url().query().unwrap();

        assert!(query.contains("pretty=true"));
        assert!(query.contains("timings=true"));
        assert!(query.contains("transaction=true"));
        assert!(query.contains("queue=true"));
        assert!(query.contains("timeout=10s"));
        assert!(query.contains("level=weak"));
        assert!(query.contains("freshness=5s"));
        assert!(query.contains("freshness_strict=true"));
        assert!(query.contains("norwrandom=true"));
        assert!(query.contains("ver=1"));
        assert!(query.contains("blob_array=true"));
    }

    #[test]
    fn unit_request_options_merge_default_query_params() {
        let mut req = RequestOptions::default();
        let default_params = RequestQueryParams::from(full_query_params());

        req.merge_default_query_params(&default_params);

        assert!(req.params.is_some());

        let req_params = req.params.unwrap();

        #[cfg(feature = "fast-blob")]
        assert_eq!(req_params.0.len(), 11);
        #[cfg(not(feature = "fast-blob"))]
        assert_eq!(req_params.0.len(), 12);
    }

    #[test]
    fn unit_request_merge() {
        // Bool params
        let mut params1 = RequestQueryParams::new();
        params1
            .0
            .push(RequestQueryParam::Bool("pretty".to_string()));
        let mut params2 = RequestQueryParams::new();
        params2
            .0
            .push(RequestQueryParam::Bool("pretty".to_string()));
        params2
            .0
            .push(RequestQueryParam::Bool("timings".to_string()));

        params1.merge(params2);

        assert_eq!(params1.0.len(), 2);

        // KV params
        let mut params1 = RequestQueryParams::new();
        params1.0.push(RequestQueryParam::KV(
            "timeout".to_string(),
            "10s".to_string(),
        ));
        let mut params2 = RequestQueryParams::new();
        params2.0.push(RequestQueryParam::KV(
            "timeout".to_string(),
            "10s".to_string(),
        ));
        params2.0.push(RequestQueryParam::KV(
            "level".to_string(),
            "weak".to_string(),
        ));

        params1.merge(params2);

        assert_eq!(params1.0.len(), 2);

        // Mixed params KV and Bool
        let mut params1 = RequestQueryParams::new();
        params1.0.push(RequestQueryParam::KV(
            "timeout".to_string(),
            "10s".to_string(),
        ));
        let mut params2 = RequestQueryParams::new();
        params2
            .0
            .push(RequestQueryParam::Bool("pretty".to_string()));
        params2.0.push(RequestQueryParam::KV(
            "timeout".to_string(),
            "10s".to_string(),
        ));
        params2.0.push(RequestQueryParam::KV(
            "level".to_string(),
            "weak".to_string(),
        ));

        params1.merge(params2);

        assert_eq!(params1.0.len(), 3);

        // Mixed params Bool and KV
        let mut params1 = RequestQueryParams::new();
        params1
            .0
            .push(RequestQueryParam::Bool("pretty".to_string()));
        let mut params2 = RequestQueryParams::new();
        params2.0.push(RequestQueryParam::KV(
            "timeout".to_string(),
            "10s".to_string(),
        ));
        params2.0.push(RequestQueryParam::KV(
            "level".to_string(),
            "weak".to_string(),
        ));
        params2
            .0
            .push(RequestQueryParam::Bool("pretty".to_string()));

        params1.merge(params2);

        assert_eq!(params1.0.len(), 3);
    }

    #[test]
    fn unit_freshness_level_to_string() {
        assert_eq!(RqliteFreshnessLevel::None.to_string(), "none");
        assert_eq!(RqliteFreshnessLevel::Weak.to_string(), "weak");
        assert_eq!(RqliteFreshnessLevel::Strong.to_string(), "strong");
    }
}
