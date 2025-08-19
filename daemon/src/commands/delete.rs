use std::{error::Error, sync::Arc};

use tokio::sync::Mutex;

use crate::{AppState, processes::Process, socket::Response};

pub async fn delete(
    app_state: Arc<Mutex<AppState>>,
    arg: String,
) -> Result<Response, Box<dyn Error>> {
    let process = if let Ok(index) = arg.parse::<u32>() {
        AppState::find_process_by_id(app_state.clone(), index).await
    } else {
        AppState::find_process_by_title(app_state.clone(), arg).await
    };
    if let Some(process) = process {
        delete_process(app_state, process).await;
        Ok(Response::Successfully(Some("Process deleted successfully")))
    } else {
        Ok(Response::Error("Couldn't find the process"))
    }
}

pub async fn delete_process(app_state: Arc<Mutex<AppState>>, process: Arc<Process>) {
    process.stop().await;
    app_state.lock().await.processes.remove(&process.id);
}
