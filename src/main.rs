use std::net::SocketAddr;
use std::path::Path;
use std::sync::RwLock;

use anyhow::Result;
use config::{AppConfig, CatchAllPort};
use configr::Config;
use engine_z::*;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};

#[macro_use] extern crate lazy_static;

lazy_static! {
	static ref APP_CONFIG: AppConfig = AppConfig::load("Engine Z", false).unwrap();
	static ref PROJECTS: RwLock<Vec<String>> = RwLock::new(
		std::fs::read_dir(&APP_CONFIG.web_root)
			.unwrap()
			.map(|entry| entry.unwrap())
			.filter(|entry| entry.metadata().unwrap().is_dir())
			.map(|dir| dir.file_name().to_string_lossy().to_string())
			.filter(|dir| dir != &APP_CONFIG.default_project)
			.collect()
	);
}

#[tokio::main]
async fn main() -> Result<()> {
	println!(
		"Starting Engine-Z on {} watching directories in {} with {} acting as the default",
		APP_CONFIG.ip,
		APP_CONFIG.web_root.display(),
		APP_CONFIG.default_project
	);
	let mut handles: Vec<JoinHandle<()>> = Vec::new();
	let mut port_counter: u16 = APP_CONFIG.port_range.start;

	let make_svc = make_service_fn(move |_| async move {
		Ok::<_, hyper::Error>(service_fn(move |req| async move { catch_all_handler(req).await }))
	});
	let handle = tokio::runtime::Handle::current();
	if let Some(addr) = match APP_CONFIG.catch_all {
		CatchAllPort::Set(port) => Some(SocketAddr::new(APP_CONFIG.ip, port)),
		CatchAllPort::First => {
			port_counter += 1;
			Some(SocketAddr::new(APP_CONFIG.ip, APP_CONFIG.port_range.start))
		},
		CatchAllPort::Last => Some(SocketAddr::new(APP_CONFIG.ip, APP_CONFIG.port_range.end)),
		CatchAllPort::None => None,
	} {
		handles.push(handle.spawn(async move {
			println!("Running catch all server on port {}", &addr.port());
			if let Err(e) = Server::bind(&addr).serve(make_svc).await {
				eprintln!("Catch all server encountered an error: {}", e);
			}
		}));
	};
	let make_svc = make_service_fn(move |_| async move {
		Ok::<_, hyper::Error>(service_fn(move |req| async move {
			req_handler(req, &APP_CONFIG.default_project).await
		}))
	});
	handles.push(handle.spawn(async move {
		let addr = SocketAddr::new(APP_CONFIG.ip, port_counter);
		println!("Running default server on port {}", &addr.port());
		if let Err(e) = Server::bind(&addr).serve(make_svc).await {
			eprintln!("Default server encountered an error: {}", e);
		}
	}));
	let projects = PROJECTS.read().unwrap();
	for project in projects.iter().cloned() {
		port_counter += 1;

		handles.push(handle.spawn(async move {
			let project = project.clone();
			let addr = SocketAddr::new(APP_CONFIG.ip, port_counter);
			println!("Running {} server on port {}", &project, &addr.port());
			let make_svc = make_service_fn(move |_| {
				let project = project.clone();
				async move {
					Ok::<_, hyper::Error>(service_fn(move |req| {
						let project = project.clone();
						async move { req_handler(req, &project).await }
					}))
				}
			});
			if let Err(e) = Server::bind(&addr).serve(make_svc).await {
				eprintln!("server encountered an error: {}", e);
			}
		}));
	}
	futures::future::join_all(handles).await;
	Ok(())
}

use path_absolutize::Absolutize;
use tokio::task::JoinHandle;

fn is_path_traversal<P>(path: P) -> bool
where
	P: AsRef<Path>,
{
	path.as_ref()
		.absolutize()
		.unwrap()
		.starts_with(&APP_CONFIG.web_root)
}

async fn catch_all_handler(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
	let projects = PROJECTS.read().unwrap();
	let uri_path = req.uri().path()[1..].to_owned();
	let (project, file) = match uri_path.split_once('/') {
		Some(v) => v,
		None =>
			if projects.contains(&uri_path) {
				(uri_path.as_str(), "index.html")
			} else if uri_path.is_empty() {
				(APP_CONFIG.default_project.as_str(), "index.html")
			} else {
				(APP_CONFIG.default_project.as_str(), uri_path.as_str())
			},
	};
	let mut path = APP_CONFIG.web_root.clone();
	if PROJECTS.read().unwrap().contains(&project.to_owned()) {
		path.push(project);
		path.push(file);
	} else {
		path.push(&APP_CONFIG.default_project);
		path.push(file);
	};
	if is_path_traversal(&path) {
		return Ok(Response::builder()
			.status(400)
			.body("Bad Request".into())
			.unwrap());
	};
	Ok(Response::new(Body::from(path.to_string_lossy().to_string())))
}

async fn req_handler<S: AsRef<str>>(
	req: Request<Body>,
	project: S,
) -> Result<Response<Body>, hyper::Error> {
	let mut path = APP_CONFIG.web_root.clone();
	path.push(project.as_ref());
	path.push(&req.uri().path()[1..]);
	Ok(Response::new(Body::from(path.to_string_lossy().to_string())))
}
