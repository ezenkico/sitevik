use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    time::Duration,
};

use actix_web::{
    App, HttpServer,
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
    std::fs::write(root.path().join("encoded name.txt"), "encoded").unwrap();
    std::fs::write(root.path().join(".site-meta"), "hidden").unwrap();
    std::fs::create_dir(root.path().join("private")).unwrap();
    std::fs::write(root.path().join("private/secret.txt"), "secret").unwrap();
    std::fs::create_dir(root.path().join("empty")).unwrap();
    root
}

async fn transport_request(address: SocketAddr, method: &str, path: &str) -> Vec<u8> {
    let request =
        format!("{method} {path} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n");

    actix_web::rt::task::spawn_blocking(move || {
        let mut stream = TcpStream::connect_timeout(&address, Duration::from_secs(5)).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        stream.write_all(request.as_bytes()).unwrap();

        let mut response = Vec::new();
        stream.read_to_end(&mut response).unwrap();
        response
    })
    .await
    .unwrap()
}

fn split_transport_response(response: &[u8]) -> (&str, &[u8]) {
    let separator = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .unwrap();
    let headers = std::str::from_utf8(&response[..separator]).unwrap();
    (headers, &response[separator + 4..])
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
async fn serves_valid_percent_encoded_filenames() {
    let root = fixture();
    let app = test::init_service(App::new().service(static_files(root.path().into(), false))).await;

    let response = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/encoded%20name.txt")
            .to_request(),
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(test::read_body(response).await, "encoded");
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
async fn head_responses_have_empty_bodies_on_the_http_transport() {
    let root = fixture();
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let address = listener.local_addr().unwrap();
    let server_root = root.path().to_path_buf();
    let server =
        HttpServer::new(move || App::new().service(static_files(server_root.clone(), false)))
            .listen(listener)
            .unwrap()
            .run();
    let handle = server.handle();
    let task = actix_web::rt::spawn(server);

    let response = transport_request(address, "HEAD", "/assets/app.js").await;

    handle.stop(true).await;
    task.await.unwrap().unwrap();

    let (headers, body) = split_transport_response(&response);
    assert!(headers.starts_with("HTTP/1.1 200"));
    assert!(body.is_empty());
}

#[actix_web::test]
async fn rejects_non_get_and_head_methods() {
    let root = fixture();
    let app = test::init_service(App::new().service(static_files(root.path().into(), false))).await;

    let response = test::call_service(&app, test::TestRequest::post().uri("/").to_request()).await;

    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
}

#[actix_web::test]
async fn rejects_non_get_and_head_methods_before_path_validation() {
    let root = fixture();
    let app = test::init_service(App::new().service(static_files(root.path().into(), false))).await;

    let response =
        test::call_service(&app, test::TestRequest::post().uri("/%00").to_request()).await;

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

#[actix_web::test]
async fn decoded_dot_segments_return_empty_not_found_responses() {
    let root = fixture();
    let app = test::init_service(App::new().service(static_files(root.path().into(), true))).await;

    for uri in ["/.", "/%2e", "/./..."] {
        let response =
            test::call_service(&app, test::TestRequest::get().uri(uri).to_request()).await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND, "{uri}");
        assert!(test::read_body(response).await.is_empty(), "{uri}");
    }
}

#[actix_web::test]
async fn server_remains_responsive_after_dot_segment_requests() {
    let root = fixture();
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let address = listener.local_addr().unwrap();
    let server_root = root.path().to_path_buf();
    let server =
        HttpServer::new(move || App::new().service(static_files(server_root.clone(), true)))
            .listen(listener)
            .unwrap()
            .run();
    let handle = server.handle();
    let task = actix_web::rt::spawn(server);

    for path in ["/.", "/%2e", "/./..."] {
        let response = transport_request(address, "GET", path).await;
        let (headers, body) = split_transport_response(&response);
        assert!(headers.starts_with("HTTP/1.1 404"), "{path}: {headers}");
        assert!(body.is_empty(), "{path}");
    }

    let response = transport_request(address, "GET", "/").await;

    handle.stop(true).await;
    task.await.unwrap().unwrap();

    let (headers, body) = split_transport_response(&response);
    assert!(headers.starts_with("HTTP/1.1 200"), "{headers}");
    assert_eq!(body, b"root");
}

#[actix_web::test]
async fn actix_invalid_paths_return_empty_not_found_responses() {
    let root = fixture();
    let app = test::init_service(App::new().service(static_files(root.path().into(), true))).await;

    for uri in [
        "/assets%2Fapp.js",
        "/malformed%2",
        "/malformed%GG",
        "/%FF",
        "/%2Areserved",
        "/reserved%3A",
        "/reserved%3C",
        "/reserved%3E",
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
