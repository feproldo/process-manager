use std::{
    error::Error,
    path::{Path, PathBuf},
    sync::Arc,
};

use tokio::{fs, sync::Mutex};

use crate::{
    AppState,
    processes::{ECOSYSTEM_NAME, Ecosystem, Process, ProcessState, ProcessStatus},
    socket::Response,
};

pub async fn start(
    app_state: Arc<Mutex<AppState>>,
    arg: String,
) -> Result<Response, Box<dyn Error>> {
    let process = if let Ok(index) = arg.parse::<u32>() {
        AppState::find_process_by_id(app_state.clone(), index).await
    } else {
        let by_title = AppState::find_process_by_title(app_state.clone(), arg.clone()).await;
        if let None = by_title {
            AppState::find_process_by_path(app_state.clone(), arg.clone()).await
        } else {
            by_title
        }
    };
    if let Some(process) = process {
        if !process.exists().await {
            tokio::spawn(async move {
                process.run().await;
            });
            Ok(Response::Successfully(Some("Process started")))
        } else {
            Ok(Response::Error("Process already started"))
        }
    } else {
        start_new_process(app_state, arg).await
    }
}

pub async fn start_new_process(
    app_state: Arc<Mutex<AppState>>,
    arg: String,
) -> Result<Response, Box<dyn Error>> {
    let ecosystem = match Ecosystem::from_path(PathBuf::from(&arg)).await {
        Ok(ecosystem) => ecosystem,
        Err(_) => return Ok(Response::Error("The ecosystem file could not be found")),
    };
    let process = {
        let mut app_state_guard = app_state.lock().await;
        let id = app_state_guard.id;
        app_state_guard.id += 1;
        let process = Arc::new(Process::init(id, ecosystem, arg));
        app_state_guard.processes.insert(id, process.clone());
        process
    };
    tokio::spawn(async move {
        process.run().await;
    });
    Ok(Response::Successfully(Some("Process started")))
}
