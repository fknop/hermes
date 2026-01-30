use std::sync::Arc;

use aide::{
    axum::{
        ApiRouter, IntoApiResponse,
        routing::{get, get_with},
    },
    openapi::OpenApi,
    scalar::Scalar,
    swagger::Swagger,
};
use axum::{Extension, Json, response::IntoResponse};

use crate::state::AppState;

pub fn docs_routes(state: Arc<AppState>) -> ApiRouter {
    aide::generate::infer_responses(true);

    let router = ApiRouter::new()
        .route(
            "/",
            get(Scalar::new("/docs/private/api.json")
                .with_title("Aide Axum")
                .axum_handler()),
        )
        .route(
            "/swagger",
            get(Swagger::new("/docs/private/api.json")
                .with_title("Aide Axum")
                .axum_handler()),
        )
        .route("/private/api.json", get(serve_docs))
        .with_state(state);

    aide::generate::infer_responses(false);

    router
}

async fn serve_docs(Extension(api): Extension<Arc<OpenApi>>) -> impl IntoApiResponse {
    Json(api).into_response()
}
