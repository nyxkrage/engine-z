use anyhow::Result;
use engine_z::config::CatchAllPort;
use engine_z::{start_catch_all_server, start_project_server, APP_CONFIG, PROJECTS};
use tokio::task::JoinHandle;

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

	let handle = tokio::runtime::Handle::current();
	if let Some(port) = match APP_CONFIG.catch_all {
		CatchAllPort::Set(port) => Some(port),
		CatchAllPort::First => {
			port_counter += 1;
			Some(APP_CONFIG.port_range.start)
		},
		CatchAllPort::Last => Some(APP_CONFIG.port_range.end),
		CatchAllPort::None => None,
	} {
		handles.push(start_catch_all_server(port, &handle));
	};

	handles.push(start_project_server(
		APP_CONFIG.default_project.clone(),
		port_counter,
		&handle,
	));
	let projects = PROJECTS.read().unwrap();
	for project in projects.iter().cloned() {
		port_counter += 1;

		handles.push(start_project_server(project.clone(), port_counter, &handle));
	}
	futures::future::join_all(handles).await;
	Ok(())
}
