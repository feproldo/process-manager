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

pub async fn status(
    app_state: Arc<Mutex<AppState>>,
    arg: String,
) -> Result<Response, Box<dyn Error>> {
    if arg.is_empty() {
        Ok(Response::Data(all_status(app_state).await))
    } else {
        Ok(Response::Successfully(Some("WIP! Contribute if you want!")))
    }
}

pub async fn all_status(app_state: Arc<Mutex<AppState>>) -> String {
    let processes: Vec<Arc<Process>> = app_state
        .lock()
        .await
        .processes
        .clone()
        .into_values()
        .collect();
    let mut output = String::from("Status of processes:\n");
    for process in processes {
        let process_status = process.state.lock().await.status.clone();
        output += format!(
            "{}. {} - {}\n",
            process.id, process.ecosystem.title, process_status
        )
        .as_str();
    }
    output
}
