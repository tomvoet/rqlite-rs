#![warn(clippy::pedantic)]
use rqlite_rs::{batch::BatchResult, prelude::*, response::RqliteResult};

mod common;

#[derive(FromRow)]
struct TestStructNamed {
    id: i32,
    name: String,
}

#[derive(FromRow)]
struct TestStructUnnamed(i32, String);

#[tokio::test]
async fn integration_ready() {
    let client = common::get_client();

    let ready = client.ready().await;

    assert!(ready);
}

#[tokio::test]
async fn integration_nodes() {
    let client = common::get_client();

    let nodes = client.nodes().await.unwrap();

    assert_eq!(nodes.len(), 1);
}

#[tokio::test]
async fn integration_leader() {
    let client = common::get_client();

    let leader = client.leader().await.unwrap();

    let leader = leader.unwrap();

    // Can't verify the leader address, as it's dynamic (because of docker)
    assert!(leader.leader);
}

#[tokio::test]
async fn integration_exec() {
    let client = common::get_client_and_reset_db().await;

    let create_query = "CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)";
    client.exec(create_query).await.unwrap();

    let insert_query = "INSERT INTO test (name) VALUES ('test')";
    let insert_result = client.exec(insert_query).await.unwrap();

    assert!(insert_result.changed());
}

#[tokio::test]
async fn integration_execute_batch() {
    let client = common::get_client_and_reset_db().await;

    let query = "CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)";
    let query2 = "INSERT INTO test (name) VALUES ('test')";

    let results = client.batch(vec![query, query2]).await.unwrap();

    assert_eq!(results.len(), 2);

    assert!(matches!(
        &results[0],
        RqliteResult::Success(BatchResult::QueryResult(_))
    ));

    let RqliteResult::Success(BatchResult::QueryResult(insert_result)) = &results[1] else {
        panic!("Expected success")
    };

    assert!(insert_result.changed());
}

#[tokio::test]
async fn integration_fetch() {
    let client = common::get_client_and_reset_db().await;

    let query = "CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)";
    let _ = client.exec(query).await.unwrap();

    let query = "INSERT INTO test (name) VALUES ('test')";
    let _ = client.exec(query).await.unwrap();

    let query = "SELECT * FROM test";
    let rows = client.fetch(query).await.unwrap();

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].get::<i32>("id").unwrap(), 1);
    assert_eq!(rows[0].get::<String>("name").unwrap(), "test");
}

#[tokio::test]
#[warn(clippy::pedantic)]
async fn integration_fetch_typed_struct_named() {
    let client = common::get_client_and_reset_db().await;

    let query = "CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)";
    let _ = client.exec(query).await.unwrap();

    let query = "INSERT INTO test (name) VALUES ('test')";
    let _ = client.exec(query).await.unwrap();

    let query = "SELECT * FROM test";
    let rows = client.fetch(query).await.unwrap();

    let tests = rows.into_typed::<TestStructNamed>().unwrap();

    assert_eq!(tests.len(), 1);
    assert_eq!(tests[0].id, 1);
    assert_eq!(tests[0].name, "test");
}

#[tokio::test]
async fn integration_fetch_typed_struct_unnamed() {
    let client = common::get_client_and_reset_db().await;

    let query = "CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)";
    let _ = client.exec(query).await.unwrap();

    let query = "INSERT INTO test (name) VALUES ('test')";
    let _ = client.exec(query).await.unwrap();

    let query = "SELECT * FROM test";
    let rows = client.fetch(query).await.unwrap();

    let tests = rows.into_typed::<TestStructUnnamed>().unwrap();

    assert_eq!(tests.len(), 1);
    assert_eq!(tests[0].0, 1);
    assert_eq!(tests[0].1, "test");
}

#[tokio::test]
async fn integration_fetch_typed_tuple() {
    let client = common::get_client_and_reset_db().await;

    let query = "CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)";
    let _ = client.exec(query).await.unwrap();

    let query = "INSERT INTO test (name) VALUES ('test')";
    let _ = client.exec(query).await.unwrap();

    let query = "SELECT * FROM test";
    let rows = client.fetch(query).await.unwrap();

    let tests = rows.into_typed::<(i32, String)>().unwrap();

    assert_eq!(tests.len(), 1);
    assert_eq!(tests[0].0, 1);
    assert_eq!(tests[0].1, "test");
}

#[tokio::test]
#[ignore]
async fn integration_transaction() {
    //let client = common::get_client_and_reset_db().await;
    //
    //let queries = vec![
    //    "CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)",
    //    "INSERT INTO test (name) VALUES ('test')",
    //];
    //
    //let result = client.transaction(queries).await.unwrap();

    //// Fix transaction return type before enabling this test
    //let result_1 = match &result[0] {
    //    RqliteResult::Success(BatchResult::QueryResult(result)) => result,
    //    _ => panic!("Expected success"),
    //};
    //
    //let result_2 = match &result[1] {
    //    RqliteResult::Success(BatchResult::QueryResult(result)) => result,
    //    _ => panic!("Expected success"),
    //};
    //
    //assert_eq!(result_1.changed(), true);
    //assert_eq!(result_2.changed(), true);
}

#[tokio::test]
async fn integration_queue() {
    let client = common::get_client_and_reset_db().await;

    let queries = vec![
        "CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)",
        "INSERT INTO test (name) VALUES ('test')",
    ];

    let result = client.queue(queries).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn integration_default_query_params() {
    let client = common::get_client_with_default_query_params();

    let query = "CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)";

    let result = client.exec(query).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn integration_request_fail() {
    let client = common::get_client_with_invalid_host();

    let query = "CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)";

    let result = client.exec(query).await;

    assert!(result.is_err());
}

// #[tokio::test]
// async fn integration_auth_success() {
//     let client = common::get_client_with_auth().await;
//
//     let query = "CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)";
//
//     let result = client.exec(query).await;
//
//     println!("{:?}", result);
//
//     assert!(result.is_ok());
// }

// #[tokio::test]
// async fn integration_auth_fail() {
//     let client = common::get_client_with_invalid_auth().await;
//
//     let query = "CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)";
//
//     let result = client.exec(query).await;
//
//     assert!(result.is_err());
// }
