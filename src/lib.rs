use std::fs::File;
use std::io::BufReader;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::RwLock;

use config::AppConfig;
use configr::Config;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use tokio::runtime::Handle;
use tokio::task::JoinHandle;
#[macro_use] extern crate lazy_static;

lazy_static! {
	pub static ref APP_CONFIG: AppConfig = AppConfig::load("Engine Z", false).unwrap();
	pub static ref PROJECTS: RwLock<Vec<String>> = RwLock::new(
		std::fs::read_dir(&APP_CONFIG.web_root)
			.unwrap()
			.map(|entry| entry.unwrap())
			.filter(|entry| entry.metadata().unwrap().is_dir())
			.map(|dir| dir.file_name().to_string_lossy().to_string())
			.filter(|dir| dir != &APP_CONFIG.default_project)
			.collect()
	);
}

pub mod config;

pub fn start_catch_all_server(
	port: u16,
	runtime_handle: &Handle,
) -> JoinHandle<()> {
	let make_svc = make_service_fn(move |_| async move {
		Ok::<_, hyper::Error>(service_fn(move |req| async move { catch_all_handler(req) }))
	});
	let addr = SocketAddr::new(APP_CONFIG.ip, port);
	runtime_handle.spawn(async move {
		println!("Running catch all server on port {}", &addr.port());
		if let Err(e) = Server::bind(&addr).serve(make_svc).await {
			eprintln!("Catch all server encountered an error: {}", e);
		}
	})
}

pub fn start_project_server(
	project: String,
	port: u16,
	runtime_handle: &Handle,
) -> JoinHandle<()> {
	runtime_handle.spawn(async move {
		let addr = SocketAddr::new(APP_CONFIG.ip, port);
		println!("Running {} server on port {}", &project, &addr.port());
		if let Err(e) = Server::bind(&addr)
			.serve(make_service_fn(move |_| {
				let project = project.clone();
				async move {
					Ok::<_, hyper::Error>(service_fn(move |req| {
						let project = project.clone();
						async move { req_handler(req, project) }
					}))
				}
			}))
			.await
		{
			eprintln!("server on port {} encountered an error: {}", port, e);
		}
	})
}

fn catch_all_handler(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
	let uri_path = &req.uri().path()[1..];
	let (mut project, file) = match uri_path.split_once('/') {
		Some((p, f)) => (p.to_owned(), f),
		None => (uri_path.to_owned(), ""),
	};
	let (project, file) = if PROJECTS.read().unwrap().contains(&project) {
		(project, file)
	} else {
		project.push('/');
		project.push_str(file);
		(APP_CONFIG.default_project.clone(), project.as_str())
	};

	let mut path = APP_CONFIG.web_root.clone();
	path.push(&project);
	path.push(&file);
	match read_file_body(&path) {
		Ok(b) => Ok(Response::new(b)),
		Err(_) => Ok(Response::builder().status(404).body("Not Found".into()).unwrap()),
	}
}

fn req_handler(
	req: Request<Body>,
	project: String,
) -> Result<Response<Body>, hyper::Error> {
	let mut path = APP_CONFIG.web_root.clone();
	path.push(project);
	path.push(&req.uri().path()[1..]);
	match read_file_body(&path) {
		Ok(b) => Ok(Response::new(b)),
		Err(_) => Ok(Response::builder().status(404).body("Not Found".into()).unwrap()),
	}
}

use std::io::Read;

fn read_file_body<P>(path: P) -> std::io::Result<Body>
where
	P: AsRef<Path>,
{
	Ok(BufReader::new(File::open(if path.as_ref().is_dir() {
		path.as_ref().join(APP_CONFIG.index_file.clone())
	} else {
		path.as_ref().to_path_buf()
	})?)
	.bytes()
	.collect::<std::io::Result<Vec<u8>>>()?
	.into())
}
