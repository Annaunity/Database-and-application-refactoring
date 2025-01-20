use axum::http::StatusCode;
use axum::http::header::CONTENT_TYPE;
use axum_test::TestServer;
use axum_test::multipart::{MultipartForm, Part};
use image_backend::model::UploadResult;
use tempfile::tempdir;

async fn create_test_image(server: &TestServer) -> UploadResult {
    let image = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-data/kitten.png"
    ))
    .unwrap();

    let res = server
        .post("/api/v1/image")
        .multipart(
            MultipartForm::new().add_part("image", Part::bytes(image).mime_type("image/png")),
        )
        .await;

    res.assert_status_ok();
    res.json()
}

#[tokio::test]
async fn create_image() {
    let data_path = tempdir().unwrap();
    let app = image_backend::build_app(data_path.path());
    let server = TestServer::new(app).unwrap();
    create_test_image(&server).await;
}

#[tokio::test]
async fn get_image() {
    let data_path = tempdir().unwrap();
    let app = image_backend::build_app(data_path.path());
    let server = TestServer::new(app).unwrap();

    let upload = create_test_image(&server).await;

    let res = server.get(&format!("/api/v1/image/{}", upload.id.0)).await;
    res.assert_status_ok();
    res.assert_header(CONTENT_TYPE, "image/png");
}

#[tokio::test]
async fn delete_image() {
    let data_path = tempdir().unwrap();
    let app = image_backend::build_app(data_path.path());
    let server = TestServer::new(app).unwrap();

    let upload = create_test_image(&server).await;

    let res = server
        .delete(&format!("/api/v1/image/{}", upload.id.0))
        .await;
    res.assert_status(StatusCode::NO_CONTENT);

    let res = server.get(&format!("/api/v1/image/{}", upload.id.0)).await;
    res.assert_status_not_found();
}
