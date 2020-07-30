use log::{error, info};

use crate::build_graph;
use crate::build_rules::Artifact;
use crate::model::Project;

use http::StatusCode;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response};
use nipper::Document;

use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

pub struct Server {
    project: Project,
    port: u16,
}

impl Server {
    pub fn from_project(project: Project) -> Server {
        Server {
            project,
            port: 4000,
        }
    }

    pub fn with_port(self, port: u16) -> Server {
        Server { port, ..self }
    }

    pub async fn listen(self) {
        let project = self.project.clone();
        let root = self.project.output_dir().clone();

        let addr = SocketAddr::from(([0, 0, 0, 0], self.port));
        let server = hyper::Server::bind(&addr).serve(make_service_fn(|_conn| {
            let root = root.clone();
            let project = project.clone();
            async {
                Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                    route(project.clone(), root.clone(), req)
                }))
            }
        }));

        info!("Server listening on http://0.0.0.0:{}", self.port);

        if let Err(e) = server.await {
            error!("server error: {}", e);
        }
    }
}

fn path_with_fallback(root: PathBuf, req: Request<Body>) -> PathBuf {
    let root = root.as_path().to_owned();
    let uri_path = &req.uri().path_and_query().unwrap().path()[1..];
    let path = root.join(uri_path);
    if path.is_dir() {
        path.join("index.html")
    } else {
        path
    }
}

async fn wait_for_changes(
    project: Project,
    _req: Request<Body>,
) -> Result<Response<Body>, Infallible> {
    info!("Awaiting for changes...");
    loop {
        tokio::time::delay_for(Duration::from_millis(100)).await;

        let build_plan = build_graph::plan_build(project.clone()).compute_diff();
        let artifacts = build_plan.execute();

        if !artifacts.is_empty() {
            let artifacts: Vec<String> = artifacts
                .into_iter()
                .map(|a| {
                    match a {
                        Artifact::File(path) => path.to_str().unwrap().to_string(),
                        _ => "".to_string(),
                    }
                    .replace(project.clone().output_dir().to_str().unwrap(), "")
                })
                .collect();
            let reply = format!("{{\"changes\": {:?} }}", artifacts);
            return Ok(Response::builder()
                .status(StatusCode::OK)
                .body(hyper::Body::from(reply))
                .unwrap());
        }
        tokio::task::yield_now().await
    }
}

async fn serve_file(root: PathBuf, req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let response = Response::builder();

    let file_path = path_with_fallback(root, req);

    let file = std::fs::read(file_path.clone());

    let file_ext = file_path
        .extension()
        .unwrap_or_else(|| std::ffi::OsStr::new(""))
        .to_str()
        .unwrap();

    let reply = match file {
        Ok(contents) => {
            let body = if file_ext == "html" {
                let contents = String::from_utf8(contents).unwrap();
                let src = include_str!("browser_reloader.js");
                let reloader = Document::from(src).html();
                let html = Document::from(&contents);
                html.select("body").append_html(reloader);
                html.html().as_bytes().to_vec()
            } else {
                contents
            };
            response
                .status(StatusCode::OK)
                .body(hyper::Body::from(body))
        }

        Err(_) => response
            .status(StatusCode::NOT_FOUND)
            .body(hyper::Body::empty()),
    };

    info!("Serving {:?}", file_path);

    Ok(reply.unwrap())
}

async fn route(
    project: Project,
    root: PathBuf,
    req: Request<Body>,
) -> Result<Response<Body>, Infallible> {
    let is_reload_path = req
        .uri()
        .path_and_query()
        .unwrap()
        .path()
        .starts_with("/___hotstuff___");

    if is_reload_path {
        wait_for_changes(project, req).await
    } else {
        serve_file(root.clone(), req).await
    }
}
