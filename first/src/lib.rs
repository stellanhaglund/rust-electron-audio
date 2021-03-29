//! This example demonstrates async/await usage with warp.

use juniper::{
    graphql_object, EmptyMutation, EmptySubscription, FieldError, GraphQLEnum, RootNode,
};
use warp::{http::Response, Filter};

use audio

#[derive(Clone, Copy, Debug)]
struct Context;
impl juniper::Context for Context {}

#[derive(Clone, Copy, Debug, GraphQLEnum)]
enum UserKind {
    Admin,
    User,
    Guest,
}

#[derive(Clone, Debug)]
struct User {
    id: i32,
    kind: UserKind,
    name: String,
}

#[graphql_object(context = Context)]
impl User {
    fn id(&self) -> i32 {
        self.id
    }

    fn kind(&self) -> UserKind {
        self.kind
    }

    fn name(&self) -> &str {
        &self.name
    }

    async fn friends(&self) -> Vec<User> {
        vec![]
        audio::main()
    }
}

#[derive(Clone, Copy, Debug)]
struct Query;

#[graphql_object(context = Context)]
impl Query {
    async fn users() -> Vec<User> {
        vec![
            User {
                id: 1,
                kind: UserKind::Admin,
                name: "user1".into(),
            },
            User {
                id: 1,
                kind: UserKind::Guest,
                name: "user2".into(),
            }            
        ]
    }

    /// Fetch a URL and return the response body text.
    async fn request(url: String) -> Result<String, FieldError> {
        Ok(reqwest::get(&url).await?.text().await?)
    }
}

type Schema = RootNode<'static, Query, EmptyMutation<Context>, EmptySubscription<Context>>;

fn schema() -> Schema {
    Schema::new(
        Query,
        EmptyMutation::<Context>::new(),
        EmptySubscription::<Context>::new(),
    )
}

#[no_mangle]
#[tokio::main]
pub async extern "C" fn start() -> i32 {
    std::env::set_var("RUST_LOG", "warp_async");
    env_logger::init();

    let log = warp::log("warp_server");

    let homepage = warp::path::end().map(|| {
        Response::builder()
            .header("content-type", "text/html")
            .body(
                "<html><h1>juniper_warp</h1><div>visit <a href=\"/graphiql\">/graphiql</a></html>",
            )
    });

    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["Content-type"])
        .allow_methods(vec!["GET", "POST", "DELETE", "OPTIONS"]);    

    log::info!("Listening on 127.0.0.1:8080");

    let state = warp::any().map(|| Context);
    let graphql_filter = juniper_warp::make_graphql_filter(schema(), state.boxed());

    warp::serve(
        warp::get()
            .and(warp::path("graphiql"))
            .and(juniper_warp::graphiql_filter("/graphql", None))
            .or(homepage)
            .or(warp::path("graphql").and(graphql_filter))
            .with(log)
            .with(cors)
    )
    .run(([127, 0, 0, 1], 8080))
    .await;

    1
}
