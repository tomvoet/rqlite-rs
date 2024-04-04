use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
};

use crate::{
    batch::BatchResult,
    error::{ClientBuilderError, RequestError},
    node::{Node, NodeResponse, RemoveNodeRequest},
    query::{self, QueryArgs, RqliteQuery},
    query_result::QueryResult,
    request::{RequestOptions, RqliteQueryParams},
    response::{RqliteResponseRaw, RqliteResult},
    select::RqliteSelectResults,
};
use reqwest::header;
use rqlite_rs_core::Row;

/// A client for interacting with a rqlite cluster.
pub struct RqliteClient {
    client: reqwest::Client,
    hosts: Arc<RwLock<VecDeque<String>>>,
}

/// A builder for creating a [`RqliteClient`].
#[derive(Default)]
pub struct RqliteClientBuilder {
    hosts: Vec<String>,
}

impl RqliteClientBuilder {
    /// Creates a new [`RqliteClientBuilder`].
    pub fn new() -> Self {
        RqliteClientBuilder::default()
    }

    /// Adds a known host to the builder.
    pub fn known_host(mut self, host: impl ToString) -> Self {
        self.hosts.push(host.to_string());
        self
    }

    /// Builds the [`RqliteClient`] with the provided hosts.
    pub fn build(self) -> Result<RqliteClient, ClientBuilderError> {
        if self.hosts.is_empty() {
            return Err(ClientBuilderError::NoHostsProvided);
        }

        let hosts = VecDeque::from(self.hosts);

        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        let client = reqwest::ClientBuilder::new()
            .timeout(std::time::Duration::from_secs(5))
            .default_headers(headers)
            .build()?;

        Ok(RqliteClient {
            client,
            hosts: Arc::new(RwLock::new(hosts)),
        })
    }
}

impl RqliteClient {
    fn shift_host(&self) {
        let mut hosts = self.hosts.write().unwrap();
        hosts.rotate_left(1);
    }

    async fn try_request(
        &self,
        options: RequestOptions,
    ) -> Result<reqwest::Response, RequestError> {
        let (mut host, host_count) = {
            let hosts = self.hosts.read().unwrap();
            (hosts[0].clone(), hosts.len())
        };

        for _ in 0..host_count {
            let req = options.to_reqwest_request(&self.client, host.as_str());

            match req.send().await {
                Ok(res) if res.status().is_success() => return Ok(res),
                Ok(res) => {
                    return Err(RequestError::ReqwestError {
                        status: res.status(),
                        body: match res.text().await {
                            Ok(body) => body,
                            Err(e) => format!("Failed to read body: {}", e),
                        },
                    });
                }
                Err(e) => self.handle_request_error(e, &mut host)?,
            }
        }

        Err(RequestError::NoAvailableHosts)
    }

    fn handle_request_error(
        &self,
        e: reqwest::Error,
        host: &mut String,
    ) -> Result<(), RequestError> {
        if e.is_connect() || e.is_timeout() {
            let previous_host = host.clone();
            self.shift_host();
            let hosts = self.hosts.read().unwrap();
            *host = hosts[0].clone();
            println!("Connection to {} failed, trying {}", previous_host, *host);
            Ok(())
        } else {
            Err(RequestError::SwitchoverWrongError(e.to_string()))
        }
    }

    async fn exec_query<T>(&self, q: query::RqliteQuery) -> Result<RqliteResult<T>, RequestError>
    where
        T: serde::de::DeserializeOwned + Clone,
    {
        let res = self
            .try_request(RequestOptions {
                endpoint: q.endpoint(),
                body: Some(
                    q.into_json()
                        .map_err(RequestError::FailedParseRequestBody)?,
                ),
                ..Default::default()
            })
            .await?;

        let body = res.text().await?;

        let response = serde_json::from_str::<RqliteResponseRaw<T>>(&body)
            .map_err(RequestError::FailedParseResponseBody)?;

        response
            .results
            .first()
            .cloned()
            .ok_or_else(|| RequestError::NoRowsReturned)
    }

    // To be implemented for different types of queries such as batch or qeued queries
    //async fn exec_many<T>(
    //    &self,
    //    qs: Vec<query::RqliteQuery>,
    //    params: impl Into<Option<Vec<RequestQueryParam>>>,
    //) -> anyhow::Result<Vec<RqliteResult<T>>>
    //where
    //    T: serde::de::DeserializeOwned + Clone,
    //{
    //    let args = QueryArgs::from(qs);
    //    let body = serde_json::to_string(&args)?;
    //
    //    let res = self.try_request("request", body, None).await?;
    //
    //    let body = res.text().await?;
    //
    //    let response = serde_json::from_str::<RqliteResponseRaw<T>>(&body)?;
    //
    //    Ok(response.results)
    //}

    /// Executes a query that returns results.
    /// Returns a vector of [`Row`]s if the query was successful, otherwise an error.
    pub async fn fetch<Q>(&self, q: Q) -> Result<Vec<Row>, RequestError>
    where
        Q: TryInto<RqliteQuery>,
        RequestError: From<Q::Error>,
    {
        let result = self
            .exec_query::<RqliteSelectResults>(q.try_into()?)
            .await?;

        match result {
            RqliteResult::Success(qr) => Ok(qr.rows()),
            RqliteResult::Error(qe) => Err(RequestError::DatabaseError(qe.error)),
        }
    }

    /// Executes a query that does not return any results.
    /// Returns the [`QueryResult`] if the query was successful, otherwise an error.
    /// Is primarily used for `INSERT`, `UPDATE`, `DELETE` and `CREATE` queries.
    pub async fn exec<Q>(&self, q: Q) -> Result<QueryResult, RequestError>
    where
        Q: TryInto<RqliteQuery>,
        RequestError: From<Q::Error>,
    {
        let query_result = self.exec_query::<QueryResult>(q.try_into()?).await?;

        match query_result {
            RqliteResult::Success(qr) => Ok(qr),
            RqliteResult::Error(qe) => Err(RequestError::DatabaseError(qe.error)),
        }
    }

    /// Executes a batch of queries.
    /// It allows sending multiple queries in a single request.
    /// This can be more efficient and reduces round-trips to the database.
    /// Returns a vector of [`RqliteResult`]s.
    /// Each result contains the result of the corresponding query in the batch.
    /// If a query fails, the corresponding result will contain an error.
    ///
    /// For more information on batch queries, see the [rqlite documentation](https://rqlite.io/docs/api/bulk-api/).
    pub async fn batch<Q>(&self, qs: Vec<Q>) -> Result<Vec<RqliteResult<BatchResult>>, RequestError>
    where
        Q: TryInto<RqliteQuery>,
        RequestError: From<Q::Error>,
    {
        let queries = qs
            .into_iter()
            .map(|q| q.try_into())
            .collect::<Result<Vec<RqliteQuery>, _>>()?;

        let batch = QueryArgs::from(queries);
        let body = serde_json::to_string(&batch).map_err(RequestError::FailedParseRequestBody)?;

        let res = self
            .try_request(RequestOptions {
                endpoint: "db/request".to_string(),
                body: Some(body),
                ..Default::default()
            })
            .await?;

        let body = res.text().await?;

        let results = serde_json::from_str::<RqliteResponseRaw<BatchResult>>(&body)
            .map_err(RequestError::FailedParseResponseBody)?
            .results;

        Ok(results)
    }

    /// Executes a transaction.
    /// A transaction is a set of queries that are executed as a single unit.
    /// If any of the queries fail, the entire transaction is rolled back.
    /// Returns a vector of [`RqliteResult`]s.
    ///
    /// For more information on transactions, see the [rqlite documentation](https://rqlite.io/docs/api/api/#transactions).
    pub async fn transaction<Q>(
        &self,
        qs: Vec<Q>,
    ) -> Result<Vec<RqliteResult<QueryResult>>, RequestError>
    where
        Q: TryInto<RqliteQuery>,
        RequestError: From<Q::Error>,
    {
        let queries = qs
            .into_iter()
            .map(|q| q.try_into())
            .collect::<Result<Vec<RqliteQuery>, _>>()?;

        let batch = QueryArgs::from(queries);
        let body = serde_json::to_string(&batch).map_err(RequestError::FailedParseRequestBody)?;

        let res = self
            .try_request(RequestOptions {
                endpoint: "db/execute".to_string(),
                body: Some(body),
                params: Some(
                    RqliteQueryParams::new()
                        .transaction()
                        .into_request_query_params(),
                ),
                ..Default::default()
            })
            .await?;

        let body = res.text().await?;

        let results = serde_json::from_str::<RqliteResponseRaw<QueryResult>>(&body)
            .map_err(RequestError::FailedParseResponseBody)?
            .results;

        Ok(results)
    }

    /// Asynchronously executes multiple queries.
    /// This results in much higher write performance.
    ///
    /// For more information on queued queries, see the [rqlite documentation](https://rqlite.io/docs/api/queued-writes/).
    pub async fn queue<Q>(&self, qs: Vec<Q>) -> Result<(), RequestError>
    where
        Q: TryInto<RqliteQuery>,
        RequestError: From<Q::Error>,
    {
        let queries = qs
            .into_iter()
            .map(|q| q.try_into())
            .collect::<Result<Vec<RqliteQuery>, _>>()?;

        let batch = QueryArgs::from(queries);
        let body = serde_json::to_string(&batch).map_err(RequestError::FailedParseRequestBody)?;

        self.try_request(RequestOptions {
            endpoint: "db/execute".to_string(),
            body: Some(body),
            params: Some(RqliteQueryParams::new().queue().into_request_query_params()),
            ..Default::default()
        })
        .await?;

        Ok(())
    }

    /// Checks if the rqlite cluster is ready.
    /// Returns `true` if the cluster is ready, otherwise `false`.
    pub async fn ready(&self) -> bool {
        match self
            .try_request(RequestOptions {
                endpoint: "readyz".to_string(),
                method: reqwest::Method::GET,
                ..Default::default()
            })
            .await
        {
            Ok(res) => res.status() == reqwest::StatusCode::OK,
            Err(_) => false,
        }
    }

    /// Retrieves the nodes in the rqlite cluster.
    /// Returns a vector of [`Node`]s.
    pub async fn nodes(&self) -> Result<Vec<Node>, RequestError> {
        let res = self
            .try_request(RequestOptions {
                endpoint: "nodes".to_string(),
                params: Some(
                    RqliteQueryParams::new()
                        .ver("2".to_string())
                        .into_request_query_params(),
                ),
                method: reqwest::Method::GET,
                ..Default::default()
            })
            .await?;

        let body = res.text().await?;

        let response = serde_json::from_str::<NodeResponse>(&body)
            .map_err(RequestError::FailedParseResponseBody)?;

        Ok(response.nodes)
    }

    /// Retrieves current the leader of the rqlite cluster.
    /// Returns a [`Node`] if a leader is found, otherwise `None`.
    pub async fn leader(&self) -> Result<Option<Node>, RequestError> {
        let nodes = self.nodes().await?;

        Ok(nodes.into_iter().find(|n| n.leader))
    }

    /// Removes a node from the rqlite cluster.
    /// Returns Ok on success and Err in case of an error.
    pub async fn remove_node(&self, id: &str) -> Result<(), RequestError> {
        let body = serde_json::to_string(&RemoveNodeRequest { id: id.to_string() })
            .map_err(RequestError::FailedParseRequestBody)?;

        let res = self
            .try_request(RequestOptions {
                endpoint: "remove".to_string(),
                body: Some(body),
                method: reqwest::Method::DELETE,
                ..Default::default()
            })
            .await?;

        if res.status().is_success() {
            Ok(())
        } else {
            Err(RequestError::DatabaseError(format!(
                "Failed to remove node: {}",
                res.text()
                    .await
                    .map_err(RequestError::FailedReadingResponse)?
            )))
        }
    }
}
