use std::{
    collections::{HashSet, VecDeque},
    sync::{Arc, RwLock},
};

use crate::{
    batch::BatchResult, config::{self, RqliteClientConfig, RqliteClientConfigBuilder}, error::{ClientBuilderError, RequestError}, fallback::{FallbackCount, FallbackStrategy}, node::{Node, NodeResponse, RemoveNodeRequest}, query::{self, QueryArgs, RqliteQuery}, query_result::QueryResult, request::{RequestOptions, RqliteQueryParam, RqliteQueryParams}, response::{RqliteResponseRaw, RqliteResult}, select::RqliteSelectResults
};
use base64::{engine::general_purpose, Engine};
use reqwest::header;
use rqlite_rs_core::Row;

/// A client for interacting with a rqlite cluster.
pub struct RqliteClient {
    client: reqwest::Client,
    hosts: Arc<RwLock<Vec<String>>>,
    config: RqliteClientConfig,
}

/// A builder for creating a [`RqliteClient`].
#[derive(Default)]
pub struct RqliteClientBuilder {
    /// This uses a `HashSet` to ensure that no duplicate hosts are added.
    hosts: Vec<String>,
    /// The configration for the client.
    config: RqliteClientConfigBuilder,
    // The base64 encoded credentials used to make authorized requests to the Rqlite cluster
    basic_auth: Option<String>,
}

impl RqliteClientBuilder {
    /// Creates a new [`RqliteClientBuilder`].
    #[must_use]
    pub fn new() -> Self {
        RqliteClientBuilder::default()
    }

    /// Adds basic auth credentials
    #[must_use]
    pub fn auth(mut self, user: &str, password: &str) -> Self {
        self.basic_auth = Some(general_purpose::STANDARD.encode(format!("{user}:{password}")));
        self
    }

    /// Adds a known host to the builder.
    /// It is important not to add the scheme to the host.
    /// The scheme is set using the `scheme` method.
    /// The host should be in the format `hostname:port`.
    /// For example, `localhost:4001`.
    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    pub fn known_host(mut self, host: impl ToString) -> Self {
        let host_str = host.to_string();

        if self.hosts.iter().any(|h| h == &host_str) {
            eprintln!("Host {} already exists", host_str);
        } else {
            self.hosts.push(host_str);
        }
        self
    }

    /// Adds a default query parameter to the builder.
    /// The `blob_array` parameter is added by default if the `fast-blob` feature is not enabled. (see <https://rqlite.io/docs/api/api/#blob-data>)
    #[must_use]
    pub fn default_query_params(mut self, params: Vec<RqliteQueryParam>) -> Self {
        self.config = self.config.default_query_params(params);
        self
    }

    /// Sets the fallback count for the client.
    /// The fallback count is the number of times the client will try to switch to another host if the current host fails.
    #[must_use]
    pub fn fallback_count(mut self, count: FallbackCount) -> Self {
        self.config = self.config.fallback_count(count);
        self
    }

    /// Sets the fallback strategy for the client.
    /// The fallback strategy is the strategy used to switch to another host if the current host fails.
    /// The default strategy is `RoundRobin`.
    #[must_use]
    pub fn fallback_strategy(mut self, strategy: impl FallbackStrategy + 'static) -> Self {
        self.config = self.config.fallback_strategy(strategy);
        self
    }

    /// Sets the fallback persistence for the client.
    /// If set to `true`, which is the default, the client will keep using the last host that was successful.
    /// If set to `false`, the client will always try the first host in the list.
    #[must_use]
    pub fn fallback_persistence(mut self, persist: bool) -> Self {
        self.config = self.config.fallback_persistence(persist);
        self
    }

    /// Sets the scheme for the client.
    #[must_use]
    pub fn scheme(mut self, scheme: config::Scheme) -> Self {
        self.config = self.config.scheme(scheme);
        self
    }

    /// Builds the [`RqliteClient`] with the provided hosts.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - No hosts were provided
    /// - Failed to create HTTP client
    /// - Invalid authorization header
    pub fn build(self) -> Result<RqliteClient, ClientBuilderError> {
        if self.hosts.is_empty() {
            return Err(ClientBuilderError::NoHostsProvided);
        }

        let hosts = self.hosts.into_iter().collect::<Vec<String>>();

        println!("hosts: {:?}", hosts);

        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        if let Some(credentials) = self.basic_auth {
            let basic_auth_fmt = format!("Basic {credentials}");
            headers.insert(
                header::AUTHORIZATION,
                header::HeaderValue::from_str(basic_auth_fmt.as_str())?,
            );
        }

        let mut client = reqwest::ClientBuilder::new()
            .timeout(std::time::Duration::from_secs(5))
            .default_headers(headers);

        if let Some(config::Scheme::Https) = self.config.scheme {
            client = client.https_only(true);
        }

        Ok(RqliteClient {
            client: client.build()?,
            hosts: Arc::new(RwLock::new(hosts)),
            config: self.config.build(),
        })
    }
}

impl RqliteClient {
    async fn try_request(
        &self,
        mut options: RequestOptions,
    ) -> Result<reqwest::Response, RequestError> {
        let (mut host, host_count) = {
            let hosts = self.hosts.read().unwrap();
            println!("hosts: {:?}", hosts);
            (hosts[0].clone(), hosts.len())
        };

        let retry_count = self.config.fallback_count.count(host_count);

        if let Some(default_params) = &self.config.default_query_params {
            options.merge_default_query_params(default_params);
        };

        for _ in 0..retry_count {
            println!("Trying host: {}", host);
            let req = options.to_reqwest_request(&self.client, host.as_str(), &self.config.scheme);

            match req.send().await {
                Ok(res) if res.status().is_success() => return Ok(res),
                Ok(res) => match res.status() {
                    reqwest::StatusCode::UNAUTHORIZED => {
                        return Err(RequestError::Unauthorized);
                    }
                    status => {
                        return Err(RequestError::ReqwestError {
                            body: res.text().await?,
                            status,
                        });
                    }
                },
                Err(e) => self.handle_request_error(&e, &mut host)?,
            }
        }

        Err(RequestError::NoAvailableHosts)
    }

    /// Handles the error returned by the request.
    /// If the error is a connection error or a timeout, it will try to switch to another host.
    /// If the error is not a connection error or a timeout, it will return an error.
    fn handle_request_error(
        &self,
        e: &reqwest::Error,
        host: &mut String, // warum wird host gepasst? wird nicht alles über self.hosts gemacht?
    ) -> Result<(), RequestError> {
        if e.is_connect() || e.is_timeout() {
            let previous_host = host.clone();
            let mut writable_hosts = self.hosts.write().unwrap();

            let new_host = self.config.fallback_strategy.write().unwrap()
                .fallback(&mut writable_hosts, host, self.config.fallback_persistence)
                .ok_or(RequestError::NoAvailableHosts)?;

            host.clone_from(new_host);
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
            .into_iter()
            .next()
            .ok_or(RequestError::NoRowsReturned)
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
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The query could not be converted to a `RqliteQuery`
    /// - The request to the rqlite server failed
    /// - The response could not be parsed
    /// - The database returned an error
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
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The query could not be converted to a `RqliteQuery`
    /// - The request to the rqlite server failed
    /// - The response could not be parsed
    /// - The database returned an error
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
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The query could not be converted to a `RqliteQuery`
    /// - The request to the rqlite server failed
    /// - The response could not be parsed
    /// - The database returned an error
    pub async fn batch<Q>(&self, qs: Vec<Q>) -> Result<Vec<RqliteResult<BatchResult>>, RequestError>
    where
        Q: TryInto<RqliteQuery>,
        RequestError: From<Q::Error>,
    {
        let queries = qs
            .into_iter()
            .map(std::convert::TryInto::try_into)
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
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The query could not be converted to a `RqliteQuery`
    /// - The request to the rqlite server failed
    /// - The response could not be parsed
    /// - The database returned an error
    /// - The transaction could not be executed
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
            .map(std::convert::TryInto::try_into)
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
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The query could not be converted to a `RqliteQuery`
    /// - The request to the rqlite server failed
    /// - The response could not be parsed
    /// - The database returned an error
    pub async fn queue<Q>(&self, qs: Vec<Q>) -> Result<(), RequestError>
    where
        Q: TryInto<RqliteQuery>,
        RequestError: From<Q::Error>,
    {
        let queries = qs
            .into_iter()
            .map(std::convert::TryInto::try_into)
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
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The request to the rqlite server failed
    /// - The response could not be parsed
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
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The request to the rqlite server failed
    /// - The response could not be parsed
    pub async fn leader(&self) -> Result<Option<Node>, RequestError> {
        let nodes = self.nodes().await?;

        Ok(nodes.into_iter().find(|n| n.leader))
    }

    /// Removes a node from the rqlite cluster.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The request body cannot be serialized
    /// - The request to the rqlite server failed
    /// - The response indicates a failure
    /// - The response body cannot be read
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_rqlite_client_builder_success() {
        let client = RqliteClientBuilder::new()
            .known_host("http://localhost:4001")
            .scheme(config::Scheme::Http)
            .build();

        assert!(client.is_ok());
    }

    #[test]
    fn unit_rqlite_client_builder_no_hosts() {
        let client = RqliteClientBuilder::new().build();

        assert!(matches!(client, Err(ClientBuilderError::NoHostsProvided)));
    }

    #[test]
    fn unit_rqlite_client_builder_https() {
        let client = RqliteClientBuilder::new()
            .known_host("http://localhost:4001")
            .scheme(config::Scheme::Https)
            .build();

        let config = client.unwrap().config;

        assert!(matches!(config.scheme, config::Scheme::Https));
    }

    #[test]
    fn unit_rqlite_client_builder_auth() {
        let client = RqliteClientBuilder::new()
            .known_host("http://localhost:4001")
            .auth("user", "password")
            .build();

        assert!(client.is_ok());
    }

    #[test]
    fn unit_rqlite_client_builder_default_query_params() {
        let client = RqliteClientBuilder::new()
            .known_host("http://localhost:4001")
            .default_query_params(vec![RqliteQueryParam::Ver("3".to_string())])
            .build();

        let config = client.unwrap().config;

        #[cfg(feature = "fast-blob")]
        assert_eq!(config.default_query_params.unwrap().0.len(), 1);
        #[cfg(not(feature = "fast-blob"))]
        assert_eq!(config.default_query_params.unwrap().0.len(), 2);
    }

    #[test]
    fn unit_rqlite_client_builder_default_scheme() {
        let client = RqliteClientBuilder::new()
            .known_host("http://localhost:4001")
            .build();

        let config = client.unwrap().config;

        assert!(matches!(config.scheme, config::Scheme::Http));
    }
}
