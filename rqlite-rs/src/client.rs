use std::fmt::Debug;

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
    hosts: Vec<&'static str>,
    active_host: &'static str,
}

/// A builder for creating a [`RqliteClient`].s
pub struct RqliteClientBuilder {
    hosts: Vec<&'static str>,
}

impl RqliteClientBuilder {
    /// Creates a new [`RqliteClientBuilder`].
    pub fn new() -> Self {
        RqliteClientBuilder { hosts: Vec::new() }
    }

    /// Adds a known host to the builder.
    pub fn known_host(mut self, host: &'static str) -> Self {
        self.hosts.push(host.as_ref());
        self
    }

    /// Builds the [`RqliteClient`] with the provided hosts.
    pub fn build(self) -> anyhow::Result<RqliteClient> {
        if self.hosts.is_empty() {
            return Err(anyhow::anyhow!("No hosts provided"));
        }

        let hosts = self.hosts.clone();

        let active_host = self.hosts.first().unwrap();

        let client = reqwest::ClientBuilder::new()
            .timeout(std::time::Duration::from_secs(5))
            .build()?;

        Ok(RqliteClient {
            client,
            hosts,
            active_host,
        })
    }
}

impl RqliteClient {
    async fn exec_query<T>(&self, q: query::RqliteQuery) -> anyhow::Result<RqliteResult<T>>
    where
        T: serde::de::DeserializeOwned + Clone + Debug,
    {
        let url = q.url(self.active_host)?;
        let json = q.to_json()?.clone();

        let req = self.client.post(url).body(json);

        let res = req.send().await?;
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
        let url = format!("http://{}/db/request", self.active_host);
        let query_args = QueryArgs::from(queries);
        let batch = serde_json::to_string(&query_args)?;

        let req = self.client.post(&url).body(batch);
        let res = req.send().await?;
        let body = res.text().await?;

        let results = serde_json::from_str::<RqliteResponseRaw<BatchResult>>(&body)?.results;

        Ok(results)
    }

    /// Checks if the rqlite cluster is ready.
    /// Returns `true` if the cluster is ready, otherwise `false`.
    pub async fn ready(&self) -> bool {
        let url = format!("http://{}/readyz", self.active_host);
        match self.client.get(&url).send().await {
            Ok(res) => res.status() == reqwest::StatusCode::OK,
            Err(_) => false,
        }
    }

    /// Retrieves the nodes in the rqlite cluster.
    /// Returns a vector of [`Node`]s.
    pub async fn nodes(&self) -> anyhow::Result<Vec<Node>> {
        let url = format!("http://{}/nodes?ver=2", self.active_host);
        let res = self.client.get(&url).send().await?;
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
        let url = format!("http://{}/remove", self.active_host);
        let body = serde_json::to_string(&RemoveNodeRequest { id: id.to_string() })?;

        let res = self.client.delete(&url).body(body).send().await?;

        if res.status().is_success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Failed to remove node: {}",
                res.text().await?
            ))
        }
    }

    /// Returns the current active host, that is used for all requests.
    pub fn active_host(&self) -> &'static str {
        self.active_host
    }
}
