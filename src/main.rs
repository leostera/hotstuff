use crypto::digest::Digest;
use http::StatusCode;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use nipper::Document;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::{HashMap, HashSet};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "hotstuff")]
struct Opt {
    #[structopt(short = "p", long = "port", name = "PORT", default_value = "4000")]
    port: u16,

    #[structopt(short = "r", long = "root", name = "ROOT", parse(from_os_str))]
    root: PathBuf,
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
    changed_files: Arc<RwLock<HashSet<String>>>,
    _req: Request<Body>,
) -> Result<Response<Body>, Infallible> {
    let mut changes = vec![];

    loop {
        match changed_files.write() {
            Ok(mut cfs) => {
                if (*cfs).is_empty() {
                    continue;
                } else {
                    let paths = (*cfs).clone();
                    for path in paths.into_iter() {
                        changes.push(path);
                    }
                    *cfs = HashSet::new();
                    break;
                }
            }
            Err(_) => std::thread::sleep(std::time::Duration::from_millis(100)),
        }
    }

    let reply = format!("{{\"changes\": {:?} }}", changes);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(hyper::Body::from(reply))
        .unwrap())
}

async fn serve_file(root: PathBuf, req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let response = Response::builder();

    let file_path = path_with_fallback(root, req);

    let file = std::fs::read(file_path.clone());

    let file_ext = file_path
        .extension()
        .unwrap_or(std::ffi::OsStr::new(""))
        .to_str()
        .unwrap();

    let reply = match file {
        Ok(contents) => {
            let body = if file_ext == "html" {
                let contents = String::from_utf8(contents).unwrap();
                let src = include_str!("hotstuff_reloader.js");
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

    println!("Serving {:?}", file_path);

    Ok(reply.unwrap())
}

async fn route(
    changed_files: Arc<RwLock<HashSet<String>>>,
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
        wait_for_changes(changed_files, req).await
    } else {
        serve_file(root.clone(), req).await
    }
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();

    let root = std::fs::canonicalize(opt.root.clone()).unwrap();

    let changed_files: Arc<RwLock<HashSet<String>>> = Arc::new(RwLock::new(HashSet::new()));

    let file_hashes: Arc<RwLock<HashMap<String, String>>> = Arc::new(RwLock::new(HashMap::new()));
    let changed_files_watcher = changed_files.clone();
    let mut watcher: RecommendedWatcher = notify::Watcher::new_immediate(move |res| {
        let opt = Opt::from_args();
        let root = std::fs::canonicalize(opt.root.clone()).unwrap();
        let root_str = root.to_str().unwrap().clone();
        match res {
            Ok(notify::event::Event {
                kind: _kind,
                paths,
                attrs: _attrs,
            }) => {
                let paths: Vec<String> = paths
                    .iter()
                    .filter(|p| !p.is_dir())
                    .map(|p| p.to_str().unwrap())
                    .filter(|p| !p.ends_with("~"))
                    .filter(|p| !p.ends_with("swp"))
                    .filter(|p| !p.ends_with("tmp"))
                    .filter(|p| !p.starts_with("/."))
                    .map(String::from)
                    .collect();

                match changed_files_watcher.write() {
                    Ok(mut cfs) => {
                        for path in paths.clone() {
                            let hash = {
                                let mut digest = crypto::sha1::Sha1::new();
                                let file = std::fs::read(path.clone()).unwrap();
                                digest.input(&file);
                                digest.result_str()
                            };

                            let path = path.replace(root_str, "");

                            let mut file_hashes = file_hashes.write().unwrap();
                            match file_hashes.get(&path) {
                                Some(old_hash) => {
                                    if !hash.eq(old_hash) {
                                        println!("File changed: {:?}", path);
                                        println!("Hash changed! {:?} != {:?}", hash, old_hash);
                                        cfs.insert(path);
                                    }
                                }
                                None => {
                                    file_hashes.insert(path.clone(), hash);
                                    cfs.insert(path);
                                }
                            }
                        }
                    }
                    Err(_) => (),
                }
            }
            _ => (),
        }
    })
    .unwrap();

    watcher
        .watch(root.clone(), RecursiveMode::Recursive)
        .unwrap();

    watcher
        .configure(notify::Config::PreciseEvents(true))
        .unwrap();

    watcher
        .configure(notify::Config::NoticeEvents(true))
        .unwrap();

    println!("Watching for changes in {:?}", root);

    let addr = SocketAddr::from(([0, 0, 0, 0], opt.port));
    let server = Server::bind(&addr).serve(make_service_fn(|_conn| {
        let changed_files = changed_files.clone();
        let opt = Opt::from_args();
        let root = std::fs::canonicalize(opt.root.clone()).unwrap();
        async {
            Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                route(changed_files.clone(), root.clone(), req)
            }))
        }
    }));

    println!("Server listening on http://0.0.0.0:{}", opt.port);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
