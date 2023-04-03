use std::{sync::Arc, env, net::SocketAddr};

use auth::Login;
use axum::Router;
use common::{context::ServiceState, repository::{Repository, mongo::MongoRepository, http_repository::Registrable}};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .init();

    let mongo_uri = env::var("MONGOURI").unwrap();

    let mut state = ServiceState::new("auth".to_string());
    state.insert(Repository(Arc::new(MongoRepository::<Login>::new(&mongo_uri, "auth", "auth").await)));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let router = Router::new()
        .register::<Login>()
        .with_state(Arc::new(state));

    tracing::info!("routs {:?}", router);

    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .unwrap();
}
