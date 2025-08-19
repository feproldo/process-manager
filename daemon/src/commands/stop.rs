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

pub async fn stop(
    app_state: Arc<Mutex<AppState>>,
    arg: String,
) -> Result<Response, Box<dyn Error>> {
    let process = if let Ok(index) = arg.parse::<u32>() {
        AppState::find_process_by_id(app_state.clone(), index).await
    } else {
        AppState::find_process_by_title(app_state.clone(), arg).await
    };
    if let Some(process) = process {
        stop_process(process).await
    } else {
        Ok(Response::Error("Couldn't find the process"))
    }
}

async fn stop_process(process: Arc<Process>) -> Result<Response, Box<dyn Error>> {
    process.stop().await;
    Ok(Response::Successfully(Some("The process is stopped")))
}
