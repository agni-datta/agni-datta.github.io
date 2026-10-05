//! Local HTTP serving and source watching.
use crate::build;
use anyhow::{Context, Result, anyhow, bail};
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::io::Cursor;
use std::path::{Component, Path, PathBuf};
use std::thread;
use std::time::{Duration, SystemTime};
use tiny_http::{Header, Response, Server, StatusCode};

pub(super) fn serve(root: &Path, arguments: &[String]) -> Result<()> {
    let port = parse_serve_port(arguments)?;
    let address = format!("127.0.0.1:{port}");
    build::build(root)?;
    let watch_root = root.to_path_buf();
    thread::spawn(move || watch_sources(&watch_root));
    let server = Server::http(&address).map_err(|error| anyhow!(error.to_string()))?;
    println!("serving http://localhost:{port}; press Ctrl-C to stop");
    for request in server.incoming_requests() {
        let path = public_path(root, request.url());
        let (status, bytes, content_type) =
            match path.and_then(|path| read_public_file(root, &path)) {
                Ok((bytes, content_type)) => (StatusCode(200), bytes, content_type),
                Err(_) => {
                    let fallback = root.join("public/404.html");
                    let bytes = fs::read(fallback).unwrap_or_else(|_| b"Not found".to_vec());
                    (StatusCode(404), bytes, "text/html; charset=utf-8")
                }
            };
        let header = Header::from_bytes("Content-Type", content_type)
            .map_err(|_| anyhow!("invalid content-type header"))?;
        let content_length = bytes.len();
        let response = Response::new(
            status,
            vec![header],
            Cursor::new(bytes),
            Some(content_length),
            None,
        );
        if let Err(error) = request.respond(response) {
            eprintln!("local response failed: {error}");
        }
    }
    Ok(())
}

fn parse_serve_port(arguments: &[String]) -> Result<u16> {
    match arguments {
        [] => Ok(8000),
        [flag, value] if flag == "--port" => {
            value.parse().context("--port must be a valid TCP port")
        }
        _ => bail!("serve accepts only an optional `--port PORT` argument"),
    }
}

fn watch_sources(root: &Path) {
    let mut previous = source_snapshot(root).unwrap_or_default();
    loop {
        thread::sleep(Duration::from_millis(750));
        let current = source_snapshot(root).unwrap_or_default();
        if current != previous {
            println!("source change detected; rebuilding");
            match build::build(root) {
                Ok(()) => println!("rebuild complete"),
                Err(error) => eprintln!("rebuild failed; serving the last valid build: {error:#}"),
            }
            previous = source_snapshot(root).unwrap_or(current);
        }
    }
}

fn source_snapshot(root: &Path) -> Result<BTreeMap<PathBuf, SystemTime>> {
    let mut snapshot = BTreeMap::new();
    for relative in [
        "content",
        "templates",
        "styles",
        "static",
        "crates/webapp",
        "CNAME",
        "robots.txt",
    ] {
        collect_modified(&root.join(relative), &mut snapshot)?;
    }
    Ok(snapshot)
}

fn collect_modified(path: &Path, output: &mut BTreeMap<PathBuf, SystemTime>) -> Result<()> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            collect_modified(&entry?.path(), output)?;
        }
    } else if path.is_file() {
        output.insert(path.to_path_buf(), fs::metadata(path)?.modified()?);
    }
    Ok(())
}

fn public_path(root: &Path, url: &str) -> Result<PathBuf> {
    let path = url
        .split(['?', '#'])
        .next()
        .unwrap_or("/")
        .trim_start_matches('/');
    let relative = Path::new(path);
    if relative
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
        && !path.is_empty()
    {
        bail!("invalid request path");
    }
    let mut candidate = root.join("public").join(relative);
    if path.is_empty() || candidate.is_dir() || candidate.extension().is_none() {
        candidate = candidate.join("index.html");
    }
    Ok(candidate)
}

fn read_public_file(root: &Path, path: &Path) -> Result<(Vec<u8>, &'static str)> {
    let public = root.join("public");
    if !path.starts_with(&public) || !path.is_file() {
        bail!("file is outside public output or missing");
    }
    let content_type = match path.extension().and_then(OsStr::to_str).unwrap_or_default() {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "wasm" => "application/wasm",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "woff2" => "font/woff2",
        "pdf" => "application/pdf",
        "xml" => "application/xml; charset=utf-8",
        "txt" | "bib" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    };
    Ok((fs::read(path)?, content_type))
}

#[cfg(test)]
mod tests {
    use super::public_path;
    use std::path::Path;

    #[test]
    fn local_routes_map_to_static_index_files() {
        assert_eq!(
            public_path(Path::new("/workspace"), "/privacy/?x=1").unwrap(),
            Path::new("/workspace/public/privacy/index.html")
        );
    }

    #[test]
    fn local_server_rejects_parent_traversal() {
        assert!(public_path(Path::new("/workspace"), "/../secret").is_err());
    }
}
