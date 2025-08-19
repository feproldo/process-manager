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

pub async fn restart(
    app_state: Arc<Mutex<AppState>>,
    arg: String,
) -> Result<Response, Box<dyn Error>> {
    let process = if let Ok(index) = arg.parse::<u32>() {
        AppState::find_process_by_id(app_state.clone(), index).await
    } else {
        AppState::find_process_by_title(app_state.clone(), arg).await
    };
    if let Some(process) = process {
        restart_process(app_state, process).await
    } else {
        Ok(Response::Error("Couldn't find the process"))
    }
}

async fn restart_process(
    app_state: Arc<Mutex<AppState>>,
    process: Arc<Process>,
) -> Result<Response, Box<dyn Error>> {
    let new_ecosystem = match Ecosystem::from_path(PathBuf::from(process.path.clone())).await {
        Ok(new_ecosystem) => new_ecosystem,
        Err(_) => return Ok(Response::Error("Couldn't get new ecosystem file")),
    };
    process.stop().await;
    let process = Arc::new(Process::init(
        process.id,
        new_ecosystem,
        process.path.clone(),
    ));
    let mut app_state_guard = app_state.lock().await;
    app_state_guard
        .processes
        .insert(process.id, process.clone());
    tokio::spawn(async move {
        let _ = process.run();
    });
    Ok(Response::Successfully(Some(
        "Rebooted with a new ecosystem file. Don't forget to make a backup:\n\tfpm backup",
    )))
}
