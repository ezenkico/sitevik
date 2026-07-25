use actix_web::{
    App,
    http::{Method, StatusCode, header},
    test,
};
use sitevik::static_files;
use tempfile::TempDir;
use toml::Value;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

fn fixture() -> TempDir {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("index.html"), "root").unwrap();
    std::fs::create_dir(root.path().join("about")).unwrap();
    std::fs::write(root.path().join("about/index.html"), "about").unwrap();
    std::fs::create_dir(root.path().join("assets")).unwrap();
    std::fs::write(root.path().join("assets/app.js"), "console.log('ok')").unwrap();
    std::fs::write(root.path().join(".site-meta"), "hidden").unwrap();
    std::fs::create_dir(root.path().join("private")).unwrap();
    std::fs::write(root.path().join("private/secret.txt"), "secret").unwrap();
    std::fs::create_dir(root.path().join("empty")).unwrap();
    root
}

#[actix_web::test]
async fn serves_root_index() {
    let root = fixture();
    let app = test::init_service(App::new().service(static_files(root.path().into(), false))).await;

    let response = test::call_service(&app, test::TestRequest::get().uri("/").to_request()).await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(test::read_body(response).await, "root");
}

#[actix_web::test]
async fn serves_directory_index_without_a_redirect() {
    let root = fixture();
    let app = test::init_service(App::new().service(static_files(root.path().into(), false))).await;

    for uri in ["/about", "/about/"] {
        let response =
            test::call_service(&app, test::TestRequest::get().uri(uri).to_request()).await;

        assert_eq!(response.status(), StatusCode::OK, "{uri}");
        assert!(response.headers().get(header::LOCATION).is_none(), "{uri}");
        assert_eq!(test::read_body(response).await, "about", "{uri}");
    }
}

#[actix_web::test]
async fn serves_javascript_with_a_javascript_content_type() {
    let root = fixture();
    let app = test::init_service(App::new().service(static_files(root.path().into(), false))).await;

    let response = test::call_service(
        &app,
        test::TestRequest::get().uri("/assets/app.js").to_request(),
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    assert!(
        response
            .headers()
            .get(header::CONTENT_TYPE)
            .unwrap()
            .to_str()
            .unwrap()
            .contains("javascript")
    );
}

#[actix_web::test]
async fn serves_hidden_files() {
    let root = fixture();
    let app = test::init_service(App::new().service(static_files(root.path().into(), false))).await;

    let response = test::call_service(
        &app,
        test::TestRequest::get().uri("/.site-meta").to_request(),
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(test::read_body(response).await, "hidden");
}

#[actix_web::test]
async fn does_not_list_directories_without_indexes() {
    let root = fixture();
    let app = test::init_service(App::new().service(static_files(root.path().into(), false))).await;

    let response =
        test::call_service(&app, test::TestRequest::get().uri("/empty").to_request()).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert!(test::read_body(response).await.is_empty());
}

#[actix_web::test]
async fn head_requests_receive_the_file_response_headers() {
    let root = fixture();
    let app = test::init_service(App::new().service(static_files(root.path().into(), false))).await;

    let response = test::call_service(
        &app,
        test::TestRequest::default()
            .method(Method::HEAD)
            .uri("/assets/app.js")
            .to_request(),
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.headers().contains_key(header::CONTENT_TYPE));
}

#[actix_web::test]
async fn rejects_non_get_and_head_methods() {
    let root = fixture();
    let app = test::init_service(App::new().service(static_files(root.path().into(), false))).await;

    let response = test::call_service(&app, test::TestRequest::post().uri("/").to_request()).await;

    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
}

#[actix_web::test]
async fn spa_disabled_does_not_serve_missing_routes() {
    let root = fixture();
    let app = test::init_service(App::new().service(static_files(root.path().into(), false))).await;

    let response = test::call_service(
        &app,
        test::TestRequest::get().uri("/dashboard").to_request(),
    )
    .await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert!(test::read_body(response).await.is_empty());
}

#[actix_web::test]
async fn spa_enabled_serves_missing_routes() {
    let root = fixture();
    let app = test::init_service(App::new().service(static_files(root.path().into(), true))).await;

    for uri in ["/dashboard", "/nested/profile"] {
        let response =
            test::call_service(&app, test::TestRequest::get().uri(uri).to_request()).await;

        assert_eq!(response.status(), StatusCode::OK, "{uri}");
        assert_eq!(test::read_body(response).await, "root", "{uri}");
    }
}

#[actix_web::test]
async fn spa_enabled_does_not_serve_missing_final_file_segments() {
    let root = fixture();
    let app = test::init_service(App::new().service(static_files(root.path().into(), true))).await;

    let response = test::call_service(
        &app,
        test::TestRequest::get().uri("/missing.js").to_request(),
    )
    .await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert!(test::read_body(response).await.is_empty());
}

#[actix_web::test]
async fn spa_enabled_classifies_only_the_final_path_segment() {
    let root = fixture();
    let app = test::init_service(App::new().service(static_files(root.path().into(), true))).await;

    let response = test::call_service(
        &app,
        test::TestRequest::get().uri("/assets/missing").to_request(),
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(test::read_body(response).await, "root");
}

#[actix_web::test]
async fn spa_enabled_without_a_root_index_returns_not_found() {
    let root = fixture();
    std::fs::remove_file(root.path().join("index.html")).unwrap();
    let app = test::init_service(App::new().service(static_files(root.path().into(), true))).await;

    let response = test::call_service(
        &app,
        test::TestRequest::get().uri("/dashboard").to_request(),
    )
    .await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert!(test::read_body(response).await.is_empty());
}

#[actix_web::test]
async fn traversal_requests_never_serve_files_or_the_spa_document() {
    let root = fixture();
    let app = test::init_service(App::new().service(static_files(root.path().into(), true))).await;

    for uri in [
        "/../private/secret.txt",
        "/%2e%2e/private/secret.txt",
        "/%2E%2E/private/secret.txt",
        "/%2e%2e%2fprivate/secret.txt",
        "/%00",
    ] {
        let response =
            test::call_service(&app, test::TestRequest::get().uri(uri).to_request()).await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND, "{uri}");
        assert!(test::read_body(response).await.is_empty(), "{uri}");
    }
}

#[cfg(unix)]
#[actix_web::test]
async fn filesystem_failure_returns_internal_server_error() {
    let root = fixture();
    let unreadable = root.path().join("unreadable.txt");
    std::fs::write(&unreadable, "unreadable").unwrap();
    std::fs::set_permissions(&unreadable, std::fs::Permissions::from_mode(0o000)).unwrap();

    if std::fs::File::open(&unreadable).is_ok() {
        // Privileged test processes can bypass file permissions.
        std::fs::set_permissions(&unreadable, std::fs::Permissions::from_mode(0o644)).unwrap();
        return;
    }

    let app = test::init_service(App::new().service(static_files(root.path().into(), false))).await;
    let response = test::call_service(
        &app,
        test::TestRequest::get().uri("/unreadable.txt").to_request(),
    )
    .await;

    std::fs::set_permissions(&unreadable, std::fs::Permissions::from_mode(0o644)).unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[actix_web::test]
async fn release_profile_contains_required_optimizations() {
    let manifest =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml")).unwrap();
    let manifest: Value = manifest.parse().unwrap();
    let release_profile = manifest.get("profile").unwrap().get("release").unwrap();

    assert_eq!(
        release_profile.get("opt-level").and_then(Value::as_integer),
        Some(3)
    );
    assert_eq!(
        release_profile.get("lto").and_then(Value::as_str),
        Some("fat")
    );
    assert_eq!(
        release_profile
            .get("codegen-units")
            .and_then(Value::as_integer),
        Some(1)
    );
    assert_eq!(
        release_profile.get("strip").and_then(Value::as_bool),
        Some(true)
    );
    assert_eq!(
        release_profile.get("panic").and_then(Value::as_str),
        Some("abort")
    );
}

#[actix_web::test]
async fn release_profile_excludes_following_table_with_inline_comment() {
    let manifest = r#"
[profile.release]
opt-level = 3

[profile.dev] # build settings
lto = "fat"
"#;

    let manifest: Value = manifest.parse().unwrap();
    let release_profile = manifest.get("profile").unwrap().get("release").unwrap();

    assert_eq!(
        release_profile.get("opt-level").and_then(Value::as_integer),
        Some(3)
    );
    assert!(release_profile.get("lto").is_none());
}
