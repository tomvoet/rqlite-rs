use std::num::NonZeroU16;

use serde::Serialize;

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
    ) -> reqwest::RequestBuilder {
        let mut req = client.request(
            self.method.clone(),
            format!("http://{}/{}", host, self.endpoint),
        );

        if let Some(body) = &self.body {
            req = req.body(body.clone());
        }

        if let Some(params) = &self.params {
            req = req.query(&params.clone().into_reqwest_query());
        }

        req
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
}

#[derive(Debug, Clone, Default)]
pub(crate) struct RequestQueryParams(Vec<RequestQueryParam>);

impl RequestQueryParams {
    fn new() -> Self {
        RequestQueryParams::default()
    }

    pub(crate) fn into_reqwest_query(self) -> Vec<(String, String)> {
        self.0.into_iter().map(|p| p.into_reqwest_query()).collect()
    }
}

#[allow(dead_code)]
pub enum RqliteFreshnessLevel {
    None,
    Weak,
    Strong,
}

impl ToString for RqliteFreshnessLevel {
    fn to_string(&self) -> String {
        match self {
            RqliteFreshnessLevel::None => "none".to_string(),
            RqliteFreshnessLevel::Weak => "weak".to_string(),
            RqliteFreshnessLevel::Strong => "strong".to_string(),
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
    /// Disable RANDOM() rewriting
    NoRWRandom,
    /// Version
    Ver(String),
}

impl RqliteQueryParam {
    fn into_request_query_param(self) -> RequestQueryParam {
        match self {
            RqliteQueryParam::Pretty => RequestQueryParam::Bool("pretty".to_string()),
            RqliteQueryParam::Timings => RequestQueryParam::Bool("timings".to_string()),
            RqliteQueryParam::Transaction => RequestQueryParam::Bool("transaction".to_string()),
            RqliteQueryParam::Queue => RequestQueryParam::Bool("queue".to_string()),
            RqliteQueryParam::Timeout(t) => {
                RequestQueryParam::KV("timeout".to_string(), format!("{}s", t))
            }
            RqliteQueryParam::Level(l) => RequestQueryParam::KV("level".to_string(), l.to_string()),
            RqliteQueryParam::Freshness(f) => {
                RequestQueryParam::KV("freshness".to_string(), format!("{}s", f))
            }
            RqliteQueryParam::FreshnessStrict => {
                RequestQueryParam::Bool("freshness_strict".to_string())
            }
            RqliteQueryParam::NoRWRandom => RequestQueryParam::Bool("norwrandom".to_string()),
            RqliteQueryParam::Ver(v) => RequestQueryParam::KV("ver".to_string(), v),
        }
    }
}

pub struct RqliteQueryParams(Vec<RqliteQueryParam>);

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

    pub(crate) fn into_request_query_params(self) -> RequestQueryParams {
        let mut params = RequestQueryParams::new();

        for p in self.0 {
            params.0.push(p.into_request_query_param());
        }

        params
    }
}

impl From<RqliteQueryParams> for RequestQueryParams {
    fn from(params: RqliteQueryParams) -> Self {
        params.into_request_query_params()
    }
}
