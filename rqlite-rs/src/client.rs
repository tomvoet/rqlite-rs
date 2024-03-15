use std::fmt::Debug;

use crate::{
    query::{self},
    query_result::{QueryResult, QueryResultRaw},
    response::{RqLiteResponseRaw, RqLiteSelectResponseRawResults},
};
use rqlite_rs_core::Row;

pub struct RqLiteClient {
    client: reqwest::Client,
    hosts: Vec<&'static str>,
    active_host: &'static str,
}

pub struct RqLiteClientBuilder {
    hosts: Vec<&'static str>,
}

impl RqLiteClientBuilder {
    pub fn new(main_host: &'static str) -> Self {
        RqLiteClientBuilder {
            hosts: vec![main_host],
        }
    }

    pub fn known_host(&mut self, host: &'static str) -> &Self {
        self.hosts.push(host);
        self
    }

    pub fn build(self) -> anyhow::Result<RqLiteClient> {
        if self.hosts.is_empty() {
            return Err(anyhow::anyhow!("No hosts provided"));
        }

        let hosts = self.hosts.clone();

        let client = reqwest::ClientBuilder::new()
            .timeout(std::time::Duration::from_secs(5))
            .build()?;

        Ok(RqLiteClient {
            client,
            hosts: self.hosts,
            active_host: hosts.first().unwrap(),
        })
    }
}

impl RqLiteClient {
    async fn exec_query<T>(&self, q: query::RqLiteQuery) -> anyhow::Result<QueryResultRaw<T>>
    where
        T: serde::de::DeserializeOwned + Clone + Debug,
    {
        let url = q.url(self.active_host)?;
        let json = q.to_json()?.clone();

        let req = self.client.post(url).body(json);

        let res = req.send().await?;
        let body = res.text().await?;

        let response = serde_json::from_str::<RqLiteResponseRaw<T>>(&body)?;

        response
            .results
            .first()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No results found in response: {}", body))
    }

    pub async fn fetch(&self, q: query::RqLiteQuery) -> anyhow::Result<Vec<Row>> {
        let result = self.exec_query::<RqLiteSelectResponseRawResults>(q).await?;

        match result {
            QueryResultRaw::Success(qr) => qr.rows(),
            QueryResultRaw::Error(qe) => Err(anyhow::anyhow!(qe.error)),
        }
    }

    pub async fn exec(&self, q: query::RqLiteQuery) -> anyhow::Result<QueryResult> {
        let query_result = self.exec_query::<QueryResult>(q).await?;

        match query_result {
            QueryResultRaw::Success(qr) => Ok(qr),
            QueryResultRaw::Error(qe) => Err(anyhow::anyhow!(qe.error)),
        }
    }
}
