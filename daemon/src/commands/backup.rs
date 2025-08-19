use std::{
    error::Error,
    path::{Path, PathBuf},
    sync::Arc,
};

use tokio::{fs, sync::Mutex};

use crate::{AppState, backup, processes::Process, socket::Response};

pub async fn backup(
    app_state: Arc<Mutex<AppState>>,
    _arg: String,
) -> Result<Response, Box<dyn Error>> {
    let (processes, max_id) = {
        let app_state_guard = app_state.lock().await;
        (app_state_guard.processes.clone(), app_state_guard.id)
    };
    let max_id = processes.keys().max().map_or(max_id, |id| id.clone());
    let processes_vec: Vec<Arc<Process>> = processes.values().cloned().collect();

    match backup::write_backup(max_id, processes_vec).await {
        Ok(_) => Ok(Response::Successfully(Some("Backup saved successfully"))),
        Err(_) => Ok(Response::Error("An error occurred while saving the backup")),
    }
}
