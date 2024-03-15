use serde::{Deserialize, Serialize};

/// A node in the rqlite cluster.
#[derive(Debug, Deserialize)]
pub struct Node {
    /// The unique identifier for the node.
    pub id: String,
    /// The address of the node's API.
    pub api_addr: String,
    /// The address of the node's Raft service.
    #[serde(rename = "addr")]
    pub raft_addr: String,
    ///If the node is a voter.
    pub voter: bool,
    /// If the node is reachable.
    pub reachable: bool,
    /// If the node is the leader.
    pub leader: bool,
    /// Latency to the node.
    pub time: f64,
    /// If there was an error reaching the node,
    /// this will contain the error message.
    pub error: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct NodeResponse {
    pub(crate) nodes: Vec<Node>,
}

#[derive(Serialize)]
pub(crate) struct RemoveNodeRequest {
    pub(crate) id: String,
}
