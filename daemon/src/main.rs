use std::{borrow::Cow, collections::HashMap, error::Error, sync::Arc};

use directories::ProjectDirs;
use tokio::sync::Mutex;

mod backup;
mod commands;
mod processes;
mod socket;

#[derive(Debug, Clone)]
pub struct AppState {
    pub id: u32,
    pub processes: HashMap<u32, Arc<processes::Process>>,
}

impl AppState {
    pub fn new(id: u32, processes: HashMap<u32, Arc<processes::Process>>) -> Self {
        Self { id, processes }
    }

    pub fn default() -> Self {
        Self::new(0, HashMap::new())
    }

    pub async fn find_process_by_id(
        // need to improve this peace of shitcode
        this: Arc<Mutex<Self>>,
        id: u32,
    ) -> Option<Arc<processes::Process>> {
        let app_state_guard = this.lock().await;
        app_state_guard.processes.get(&id).cloned()
    }

    pub async fn find_process_by_title(
        this: Arc<Mutex<Self>>,
        title: String,
    ) -> Option<Arc<processes::Process>> {
        // todo: dry pattern broken :(. Refactor
        let process = {
            let guard = this.lock().await;
            let processes: Vec<Arc<processes::Process>> =
                guard.processes.clone().into_values().collect();
            let mut id: Option<u32> = None;
            for process in processes.into_iter() {
                if process.ecosystem.title == title {
                    id = Some(process.id);
                }
            }
            match id {
                Some(id) => guard.processes.get(&id).cloned(),
                None => None,
            }
        };
        process
    }

    pub async fn find_process_by_path(
        // todo: dry pattern broken :(. Refactor
        this: Arc<Mutex<Self>>,
        path: String,
    ) -> Option<Arc<processes::Process>> {
        let process = {
            let guard = this.lock().await;
            let processes: Vec<Arc<processes::Process>> =
                guard.processes.clone().into_values().collect();
            let mut id: Option<u32> = None;
            for process in processes.into_iter() {
                if process.path == path {
                    id = Some(process.id);
                }
            }
            match id {
                Some(id) => guard.processes.get(&id).cloned(),
                None => None,
            }
        };
        process
    }
}

#[inline(always)]
pub fn project_dir() -> Option<ProjectDirs> {
    ProjectDirs::from("com", "feproldo", "process manager")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let app_state = Arc::new(Mutex::new(AppState::default()));
    let app_state_processes = app_state.clone();
    let _ = processes::start_processes(app_state_processes).await;
    let app_state_socket = app_state.clone();
    println!("{:#?}", app_state_socket.lock().await);
    // tokio::spawn(async move {
    let _ = socket::start_socket(app_state_socket).await;
    // });
    Ok(())
}
