use axum::routing::{get, post};
use axum::Router;
use cf::auth;
use cf::config::CfConfig;
use cf::user::{create_user, delete_user, find_user_by_id, find_user_by_name, get_number_of_all_users, get_user_in_page, update_user};
use cf::user_config::get_user_cfg_data;
use mongodb::{Client, Database};
use tower_http::cors::Any;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::info;
use tracing_error::ErrorLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter, Registry};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let file_appender = tracing_appender::rolling::daily("logs", "cf.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    //write in json format, if not it leads unreadable characters in the log file.
    let file_layer = fmt::Layer::default().json().with_writer(non_blocking);
    let formatting_layer = fmt::layer() /*.pretty()*/
        .with_writer(std::io::stderr);
    Registry::default()
        .with(env_filter)
        // ErrorLayer 可以让 color-eyre 获取到 span 的信息
        .with(ErrorLayer::default())
        .with(formatting_layer)
        .with(file_layer)
        .init();
    color_eyre::install().unwrap();

    info!("Start from {:?}", std::env::current_dir().unwrap());

    let config = CfConfig::load("src/config/config.toml")?;

    let client = Client::with_uri_str(config.db_url()).await?;
    let user_db = client.database("user");

    let mut app = create_app();
    app = user_router(app, &user_db);
    app = auth_router(app, &user_db);
    app = app_layer(app);

    //start http server
    let http_service_url = config.service_url();
    info!("Start service @ http://s{}", http_service_url);
    let listener = tokio::net::TcpListener::bind(&*http_service_url)
        .await
        .unwrap();
    info!("Listening on {}", http_service_url);
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

fn create_app() -> Router {
    Router::new().route("/cf/v1", get(|| async { "Hello" }))
}
fn user_router(app: Router, user_db: &Database) -> Router {
    app.route(
        "/cf/user",
        post(create_user)
            .with_state(user_db.clone())
            .put(update_user)
            .with_state(user_db.clone())
            .delete(delete_user)
            .with_state(user_db.clone())
    )
    .route(
        "/cf/user/id/:id",
        get(find_user_by_id).with_state(user_db.clone()),
    )
    .route(
        "/cf/user/name/:name",
        get(find_user_by_name).with_state(user_db.clone()),
    )
    .route(
        "/cf/user/num",
        get(get_number_of_all_users).with_state(user_db.clone()),
    )
    .route(
        "/cf/user/pagi",
        post(get_user_in_page).with_state(user_db.clone()),
    )
    .route(
        "/cf/user/cfg",
        get(get_user_cfg_data).with_state(user_db.clone()),
    )
}
fn auth_router(app: Router, user_db: &Database) -> Router {
    app.route(
        "/cf/auth", post(auth::authenticate).with_state(user_db.clone())
    )
}
fn app_layer(app: Router) -> Router {
    app.layer(
        tower_http::cors::CorsLayer::new()
            .allow_methods(Any)
            .allow_headers(Any)
            .allow_origin(Any),
    )
    .layer(
        TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::default().include_headers(true)),
    )
}
