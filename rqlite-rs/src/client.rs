use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
};

use crate::{
    batch::BatchResult,
    node::{Node, NodeResponse, RemoveNodeRequest},
    query::{self, QueryArgs},
    query_result::QueryResult,
    response::{RqliteResponseRaw, RqliteResult},
    select::RqliteSelectResults,
};
use rqlite_rs_core::Row;

/// A client for interacting with a rqlite cluster.
pub struct RqliteClient {
    client: reqwest::Client,
    hosts: Arc<Mutex<Vec<String>>>,
}

/// A builder for creating a [`RqliteClient`].s
pub struct RqliteClientBuilder {
    hosts: Vec<String>,
}

impl RqliteClientBuilder {
    /// Creates a new [`RqliteClientBuilder`].
    pub fn new() -> Self {
        RqliteClientBuilder { hosts: Vec::new() }
    }

    /// Adds a known host to the builder.
    pub fn known_host(mut self, host: impl ToString) -> Self {
        self.hosts.push(host.to_string());
        self
    }

    /// Builds the [`RqliteClient`] with the provided hosts.
    pub fn build(self) -> anyhow::Result<RqliteClient> {
        if self.hosts.is_empty() {
            return Err(anyhow::anyhow!("No hosts provided"));
        }

        let hosts = self.hosts.clone();

        let client = reqwest::ClientBuilder::new()
            .timeout(std::time::Duration::from_secs(5))
            .build()?;

        Ok(RqliteClient {
            client,
            hosts: Arc::new(Mutex::new(hosts)),
        })
    }
}

impl RqliteClient {
    fn shift_host(&self) {
        let mut hosts = self.hosts.lock().unwrap();
        let host = hosts.remove(0);
        hosts.push(host);
    }

    async fn try_request(
        &self,
        endpoint: impl AsRef<str>,
        body: impl Into<Option<String>>,
    ) -> anyhow::Result<reqwest::Response> {
        let body = body.into().unwrap_or_default();
        let (mut host, host_count) = {
            let hosts = self.hosts.lock().unwrap();
            let host = hosts[0].clone();
            (host, hosts.len())
        };

        for _ in 0..host_count {
            let url = format!("http://{}/{}", host, endpoint.as_ref());
            let req = self.client.post(&url).body(body.clone());

            match req.send().await {
                Ok(res) => return Ok(res),
                Err(e) => {
                    if e.is_connect() || e.is_timeout() {
                        let previous_host = host.clone();
                        self.shift_host();
                        let hosts = self.hosts.lock().unwrap();
                        host = hosts[0].clone();
                        println!("Connection to {} failed, trying {}", previous_host, host);
                    } else {
                        return Err(e.into());
                    }
                }
            }
        }

        Err(anyhow::anyhow!("No hosts available"))
    }

    async fn exec_query<T>(&self, q: query::RqliteQuery) -> anyhow::Result<RqliteResult<T>>
    where
        T: serde::de::DeserializeOwned + Clone,
    {
        let res = self.try_request(q.endpoint(), q.to_json()?).await?;

        let body = res.text().await?;

        let response = serde_json::from_str::<RqliteResponseRaw<T>>(&body)?;

        response
            .results
            .first()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No results found in response: {}", body))
    }

    /// Executes a query that returns results.
    /// Returns a vector of [`Row`]s if the query was successful, otherwise an error.
    pub async fn fetch(&self, q: query::RqliteQuery) -> anyhow::Result<Vec<Row>> {
        let result = self.exec_query::<RqliteSelectResults>(q).await?;

        match result {
            RqliteResult::Success(qr) => qr.rows(),
            RqliteResult::Error(qe) => Err(anyhow::anyhow!(qe.error)),
        }
    }

    /// Executes a query that does not return any results.
    /// Returns the [`QueryResult`] if the query was successful, otherwise an error.
    /// Is primarily used for `INSERT`, `UPDATE`, `DELETE` and `CREATE` queries.
    pub async fn exec(&self, q: query::RqliteQuery) -> anyhow::Result<QueryResult> {
        let query_result = self.exec_query::<QueryResult>(q).await?;

        match query_result {
            RqliteResult::Success(qr) => Ok(qr),
            RqliteResult::Error(qe) => Err(anyhow::anyhow!(qe.error)),
        }
    }

    /// Executes a batch of queries.
    /// It allows sending multiple queries in a single request.
    /// This can be more efficient and reduces round-trips to the database.
    /// Returns a vector of [`RqliteResult`]s.
    /// Each result contains the result of the corresponding query in the batch.
    /// If a query fails, the corresponding result will contain an error.
    pub async fn batch(
        &self,
        queries: Vec<query::RqliteQuery>,
    ) -> anyhow::Result<Vec<RqliteResult<BatchResult>>> {
        let batch = QueryArgs::from(queries);
        let body = serde_json::to_string(&batch)?;

        let res = self.try_request("request", body).await?;

        let body = res.text().await?;

        let results = serde_json::from_str::<RqliteResponseRaw<BatchResult>>(&body)?.results;

        Ok(results)
    }

    /// Checks if the rqlite cluster is ready.
    /// Returns `true` if the cluster is ready, otherwise `false`.
    pub async fn ready(&self) -> bool {
        match self.try_request("readyz", None).await {
            Ok(res) => res.status() == reqwest::StatusCode::OK,
            Err(_) => false,
        }
    }

    /// Retrieves the nodes in the rqlite cluster.
    /// Returns a vector of [`Node`]s.
    pub async fn nodes(&self) -> anyhow::Result<Vec<Node>> {
        let res = self.try_request("nodes?ver=2", None).await?;

        let body = res.text().await?;

        let response = serde_json::from_str::<NodeResponse>(&body)?;

        Ok(response.nodes)
    }

    /// Retrieves current the leader of the rqlite cluster.
    /// Returns a [`Node`] if a leader is found, otherwise `None`.
    pub async fn leader(&self) -> anyhow::Result<Option<Node>> {
        let nodes = self.nodes().await?;

        Ok(nodes.into_iter().find(|n| n.leader))
    }

    /// Removes a node from the rqlite cluster.
    /// Returns Ok on success and Err in case of an error.
    pub async fn remove_node(&self, id: &str) -> anyhow::Result<()> {
        let body = serde_json::to_string(&RemoveNodeRequest { id: id.to_string() })?;

        let res = self.try_request("remove", body).await?;

        if res.status().is_success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Failed to remove node: {}",
                res.text().await?
            ))
        }
    }
}
