use std::{io::ErrorKind, path::PathBuf};

use actix_files::{Files, NamedFile, PathBufWrap};
use actix_web::{
    HttpResponse,
    dev::{ServiceRequest, ServiceResponse, fn_service},
};

pub fn static_files(root: PathBuf, spa: bool) -> Files {
    Files::new("/", root.clone())
        .guard(actix_web::guard::fn_guard(|context| {
            safe_request_path(context.head().uri.path())
        }))
        .index_file("index.html")
        .use_hidden_files()
        .path_filter(|_, request| safe_request_path(request.uri.path()))
        .default_handler(fn_service(move |req| fallback(req, root.clone(), spa)))
}

async fn fallback(
    req: ServiceRequest,
    root: PathBuf,
    spa: bool,
) -> Result<ServiceResponse, actix_web::Error> {
    if !safe_request_path(req.match_info().unprocessed()) {
        return Ok(not_found(req));
    }

    let path = match PathBufWrap::parse_path(req.match_info().unprocessed(), true) {
        Ok(path) => path,
        Err(_) => return Ok(not_found(req)),
    };
    let candidate = root.join(path.as_ref());

    match std::fs::metadata(&candidate) {
        Err(error) if error.kind() == ErrorKind::NotFound => {
            serve_missing(req, root, path, spa).await
        }
        Err(_) => Ok(internal_server_error(req)),
        Ok(metadata) if metadata.is_dir() => {
            match std::fs::metadata(candidate.join("index.html")) {
                Err(error) if error.kind() == ErrorKind::NotFound => Ok(not_found(req)),
                Err(_) => Ok(internal_server_error(req)),
                Ok(_) => Ok(internal_server_error(req)),
            }
        }
        Ok(_) => Ok(internal_server_error(req)),
    }
}

async fn serve_missing(
    req: ServiceRequest,
    root: PathBuf,
    path: PathBufWrap,
    spa: bool,
) -> Result<ServiceResponse, actix_web::Error> {
    let final_component_has_extension = path
        .as_ref()
        .file_name()
        .is_some_and(|component| component.to_string_lossy().contains('.'));

    if !spa || final_component_has_extension {
        return Ok(not_found(req));
    }

    match NamedFile::open_async(root.join("index.html")).await {
        Ok(file) => {
            let (request, _) = req.into_parts();
            let response = file.into_response(&request);
            Ok(ServiceResponse::new(request, response))
        }
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(not_found(req)),
        Err(_) => Ok(internal_server_error(req)),
    }
}

fn not_found(req: ServiceRequest) -> ServiceResponse {
    req.into_response(HttpResponse::NotFound().finish())
}

fn internal_server_error(req: ServiceRequest) -> ServiceResponse {
    req.into_response(HttpResponse::InternalServerError().finish())
}

fn safe_request_path(path: &str) -> bool {
    let bytes = path.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] != b'%' {
            decoded.push(bytes[index]);
            index += 1;
            continue;
        }

        if index + 2 >= bytes.len() {
            return false;
        }

        let Some(high) = hex_value(bytes[index + 1]) else {
            return false;
        };
        let Some(low) = hex_value(bytes[index + 2]) else {
            return false;
        };
        decoded.push(high * 16 + low);
        index += 3;
    }

    let Ok(decoded) = std::str::from_utf8(&decoded) else {
        return false;
    };

    !decoded.contains('\0') && decoded.split('/').all(|segment| segment != "..")
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
