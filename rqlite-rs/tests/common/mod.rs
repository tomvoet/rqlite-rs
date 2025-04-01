use rqlite_rs::{config::Scheme, request::RqliteQueryParam, RqliteClient, RqliteClientBuilder};

pub fn get_client() -> RqliteClient {
    RqliteClientBuilder::default()
        .known_host("localhost:4001")
        .scheme(Scheme::Http)
        .build()
        .unwrap()
}

#[allow(dead_code)]
pub fn get_client_with_default_query_params() -> RqliteClient {
    RqliteClientBuilder::default()
        .known_host("localhost:4001")
        .scheme(Scheme::Http)
        .default_query_params(vec![RqliteQueryParam::Pretty])
        .build()
        .unwrap()
}

#[allow(dead_code)]
pub fn get_client_with_invalid_host() -> RqliteClient {
    RqliteClientBuilder::default()
        .known_host("localhost:4042")
        .scheme(Scheme::Http)
        .build()
        .unwrap()
}

#[allow(dead_code)]
pub async fn get_client_and_reset_db() -> RqliteClient {
    let client = get_client();

    let query = "DROP TABLE IF EXISTS test";
    let _ = client.exec(query).await.unwrap();

    client
}

// pub async fn get_client_with_auth() -> RqliteClient {
//     RqliteClientBuilder::default()
//         .known_host("localhost:4003")
//         .scheme(Scheme::Http)
//         .auth("mary", "secret2")
//         .build()
//         .unwrap()
// }

// pub async fn get_client_with_invalid_auth() -> RqliteClient {
//     RqliteClientBuilder::default()
//         .known_host("localhost:4003")
//         .scheme(Scheme::Http)
//         .auth("bob", "wrong")
//         .build()
//         .unwrap()
// }
