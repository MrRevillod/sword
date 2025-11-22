use axum::response::{IntoResponse, Response};
use axum_responses::http::HttpResponse;
use std::time::Duration;
use tower::{ServiceBuilder, util::MapResponseLayer};
use tower_http::timeout::TimeoutLayer as TowerTimeoutLayer;
use tower_layer::{Identity, Stack};

use crate::core::middlewares::ResponseFnMapper;

pub struct TimeoutLayer;

type TimeoutLayerType = (
    TowerTimeoutLayer,
    ServiceBuilder<Stack<MapResponseLayer<ResponseFnMapper>, Identity>>,
);

impl TimeoutLayer {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(d_seconds: u64) -> TimeoutLayerType {
        let timeout_layer = TowerTimeoutLayer::new(Duration::from_secs(d_seconds));

        fn timeout_mapper(response: Response) -> Response {
            if response.status().as_u16() == 408 {
                return HttpResponse::RequestTimeout().into_response();
            }

            response
        }

        let response_layer =
            ServiceBuilder::new().map_response(timeout_mapper as ResponseFnMapper);

        (timeout_layer, response_layer)
    }
}
