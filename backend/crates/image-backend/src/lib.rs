mod error;
mod globals;
pub mod model;
mod resource;

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use axum::error_handling::HandleErrorLayer;
use axum::extract::connect_info::IntoMakeServiceWithConnectInfo;
use axum::extract::{MatchedPath, Request};
use axum::http::StatusCode;
use axum::{BoxError, Router};
use model::{ImageId, UploadResult};
use reqwest::multipart::{Form, Part};
use reqwest::{Client, Response};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use crate::error::{AppJson, ErrorResponse};
use crate::globals::Globals;

async fn handle_timeout_error(err: BoxError) -> (StatusCode, AppJson<ErrorResponse>) {
    if err.is::<tower::timeout::error::Elapsed>() {
        return (
            StatusCode::REQUEST_TIMEOUT,
            AppJson(ErrorResponse::message(err)),
        );
    }

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        AppJson(ErrorResponse::message(err)),
    )
}

pub fn build_app(data_path: &Path) -> IntoMakeServiceWithConnectInfo<Router, SocketAddr> {
    let api = Router::new().nest("/image", resource::image::routes());

    let globals = Globals {
        data_path: Arc::from(data_path),
    };

    Router::new()
        .nest("/api/v1", api)
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_timeout_error))
                .timeout(Duration::from_secs(5)),
        )
        .layer(TraceLayer::new_for_http().make_span_with(|req: &Request| {
            let method = req.method();
            let uri = req.uri();

            let matched_path = req
                .extensions()
                .get::<MatchedPath>()
                .map(|matched_path| matched_path.as_str());

            tracing::debug_span!("request", %method, %uri, matched_path)
        }))
        .with_state(globals)
        .into_make_service_with_connect_info::<SocketAddr>()
}

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error(transparent)]
    Transport(#[from] reqwest::Error),
    #[error("{message} ({code})")]
    Api { code: StatusCode, message: String },
}

pub struct ImageService {
    client: Client,
    base_url: String,
}

impl ImageService {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }

    async fn check_res(res: Response) -> Result<Response, ServiceError> {
        if res.status().is_success() {
            return Ok(res);
        }

        let code = res.status();

        match res.json::<ErrorResponse>().await {
            Ok(v) => Err(ServiceError::Api {
                code,
                message: v.message,
            }),
            Err(_) => Err(ServiceError::Api {
                code,
                message: code.to_string(),
            }),
        }
    }

    pub async fn create_white_image(
        &self,
        width: u32,
        height: u32,
    ) -> Result<UploadResult, ServiceError> {
        let res = self
            .client
            .post(format!("{}/api/v1/image", self.base_url))
            .query(&[("width", width), ("height", height)])
            .send()
            .await?;
        let res = Self::check_res(res).await?;
        Ok(res.json().await?)
    }

    pub async fn create_image(
        &self,
        width: u32,
        height: u32,
        data: Vec<u8>,
    ) -> Result<UploadResult, ServiceError> {
        let res = self
            .client
            .post(format!("{}/api/v1/image", self.base_url))
            .query(&[("width", width), ("height", height)])
            .multipart(Form::new().part("image", Part::bytes(data).mime_str("image/png").unwrap()))
            .send()
            .await?;
        let res = Self::check_res(res).await?;
        Ok(res.json().await?)
    }

    pub async fn get_image(&self, id: ImageId) -> Result<Vec<u8>, ServiceError> {
        let res = self
            .client
            .get(format!("{}/api/v1/image/{}", self.base_url, id.0))
            .send()
            .await?;
        let res = Self::check_res(res).await?;
        Ok(res.bytes().await?.to_vec())
    }

    pub async fn resize_image(
        &self,
        id: ImageId,
        width: u32,
        height: u32,
    ) -> Result<UploadResult, ServiceError> {
        let res = self
            .client
            .post(format!("{}/api/v1/image/{}/resize", self.base_url, id.0))
            .query(&[("width", width), ("height", height)])
            .send()
            .await?;
        let res = Self::check_res(res).await?;
        Ok(res.json().await?)
    }

    pub async fn resize_image_fill(
        &self,
        id: ImageId,
        width: u32,
        height: u32,
    ) -> Result<UploadResult, ServiceError> {
        let res = self
            .client
            .post(format!("{}/api/v1/image/{}/resize", self.base_url, id.0))
            .query(&[
                ("width", width.to_string()),
                ("height", height.to_string()),
                ("fill", "true".to_string()),
            ])
            .send()
            .await?;
        let res = Self::check_res(res).await?;
        Ok(res.json().await?)
    }
}
