use std::{error::Error, fmt::Display, process, sync::Arc};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::{
    processes::{Ecosystem, Process, ProcessStatus},
    project_dir,
};

#[derive(Deserialize, Debug)]
pub struct Backup {
    max_id: u32,
    process: Vec<BackupProcess>,
}

#[derive(Deserialize, Debug)]
pub struct BackupProcess {
    pub id: u32,
    pub ecosystem: Ecosystem,
    pub path: String,
    pub should_stop: bool,
}

impl BackupProcess {
    pub fn new(id: u32, ecosystem: Ecosystem, path: String, should_stop: bool) -> Self {
        Self {
            id,
            ecosystem,
            path,
            should_stop,
        }
    }

    async fn from_process(process: Arc<Process>) -> Self {
        let should_stop = {
            let state = process.state.lock().await;
            state.should_stop
        };
        Self::new(
            process.id,
            process.ecosystem.clone(),
            process.path.clone(),
            should_stop,
        )
    }
}

impl Display for BackupProcess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "id={}\npath=\"{}\"\nshould_stop={}\n{}",
            self.id, self.path, self.should_stop, self.ecosystem
        )
    }
}

pub async fn load_backup() -> Vec<BackupProcess> {
    let project_dir = match project_dir() {
        Some(project_dir) => project_dir,
        None => {
            eprintln!("Can't get ProjectDir");
            process::exit(1);
        }
    };
    let data_dir = project_dir.data_dir();
    if !data_dir.exists() {
        let _ = fs::create_dir_all(data_dir).await;
    }
    if !data_dir.join("backup.toml").exists() {
        let _ = fs::write(data_dir.join("backup.toml"), "").await;
    }
    let backup_content =
        if let Ok(content) = fs::read_to_string(&data_dir.join("backup.toml")).await {
            content
        } else {
            let _ = fs::write(data_dir.join("backup.toml"), "").await;
            String::from("")
        };
    let backup: Backup = match toml::from_str(backup_content.as_str()) {
        Ok(backup) => backup,
        Err(err) => {
            println!("{}", err);
            Backup {
                max_id: 0,
                process: vec![],
            }
        }
    };
    println!("{:#?}", backup);
    backup.process
}
pub async fn write_backup(max_id: u32, processes: Vec<Arc<Process>>) -> Result<(), Box<dyn Error>> {
    let mut backup_string = format!("max_id = {}", max_id);
    for process in processes {
        let process = BackupProcess::from_process(process).await;
        backup_string = format!("{}\n[[process]]\n{}", backup_string, process);
    }

    let project_dir = match project_dir() {
        Some(project_dir) => project_dir,
        None => {
            eprintln!("Can't get ProjectDir");
            process::exit(1);
        }
    };
    let data_dir = project_dir.data_dir();

    fs::write(data_dir.join("backup.toml"), backup_string).await;
    Ok(())
}
