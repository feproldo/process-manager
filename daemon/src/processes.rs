use std::{
    env,
    error::Error,
    fmt::Display,
    path::{Path, PathBuf},
    process::{self, Stdio},
    sync::Arc,
};

use chrono::{DateTime, Utc};
use directories::ProjectDirs;
use nix::{
    sys::signal::{Signal, kill},
    unistd::Pid,
};
use notify::{Event, RecursiveMode, Watcher};
use procfs::process::all_processes;
use serde::Deserialize;
use tokio::{
    fs,
    io::{AsyncBufReadExt, BufReader},
    process::{Child, Command},
    sync::{
        Mutex,
        broadcast::{self, Receiver, Sender},
        mpsc,
    },
};

use crate::{
    backup::{self, BackupProcess},
    project_dir,
};

pub const ECOSYSTEM_NAME: &str = "pm-ecosystem.toml";

#[derive(Deserialize, Clone, Copy, Debug)]
pub enum LogsMode {
    #[serde(rename = "override")]
    Override,
    #[serde(rename = "append")]
    Append,
}

impl Display for LogsMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = match self {
            Self::Override => "override",
            Self::Append => "append",
        };
        write!(f, "{res}")
    }
}

#[derive(Deserialize, Clone, Copy, Debug)]
pub enum RestartMode {
    #[serde(rename = "on_error")]
    OnError,
    #[serde(rename = "always")]
    Always,
    #[serde(rename = "never")]
    Never,
}

impl Display for RestartMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = match self {
            Self::Never => "never",
            Self::Always => "always",
            Self::OnError => "on_error",
        };
        write!(f, "{res}")
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct Ecosystem {
    pub title: String,
    pub start: String,
    pub shell: String,
    pub description: Option<String>,
    pub restart: Option<RestartMode>,
    pub logs: Option<String>,
    pub logs_mode: Option<LogsMode>,
    pub watch: Option<Vec<String>>,
}

impl Display for Ecosystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const REPLACE_FROM: &str = "\"";
        const REPLACE_TO: &str = "\\\"";
        let mut result = format!(
            "ecosystem = {{ title=\"{}\", start=\"{}\", shell=\"{}\"",
            self.title.replace(REPLACE_FROM, REPLACE_TO),
            self.start.replace(REPLACE_FROM, REPLACE_TO),
            self.shell.replace(REPLACE_FROM, REPLACE_TO)
        );
        if let Some(description) = self.description.clone() {
            result = format!(
                "{result}, description=\"{}\"",
                description.replace(REPLACE_FROM, REPLACE_TO)
            );
        };
        if let Some(restart) = self.restart {
            result = format!("{result}, restart=\"{}\"", restart);
        };
        if let Some(logs) = self.logs.clone() {
            result = format!(
                "{result}, logs=\"{}\"",
                logs.replace(REPLACE_FROM, REPLACE_TO)
            );
        };
        if let Some(logs_mode) = self.logs_mode {
            result = format!("{result}, logs_mode=\"{}\"", logs_mode);
        };
        if let Some(watch) = self.watch.clone() {
            result = format!("{result}, watch=[",);
            for (index, el) in watch.iter().enumerate() {
                if index == 0 {
                    result = format!("{result}\"{}\"", el.replace(REPLACE_FROM, REPLACE_TO));
                } else {
                    result = format!("{result}, \"{}\"", el.replace(REPLACE_FROM, REPLACE_TO));
                }
            }
            result = format!("{result}]")
        };
        result = format!("{result} }}");
        write!(f, "{result}")
    }
}

impl Ecosystem {
    pub async fn from_path(path: PathBuf) -> Result<Self, Box<dyn Error>> {
        let ecosystem_path =
            match fs::read_to_string(PathBuf::from(path).join(ECOSYSTEM_NAME)).await {
                Ok(file) => file,
                Err(err) => return Err(Box::new(err)),
            };
        let ecosystem = toml::from_str::<Ecosystem>(&ecosystem_path.as_str());
        match ecosystem {
            Ok(ecosystem_file) => Ok(ecosystem_file),
            Err(err) => return Err(Box::new(err)),
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
pub enum ProcessStatus {
    Initialized,
    Started,
    Starting,
    Paused,
    Pausing,
    Error,
    NotFound,
}

impl Display for ProcessStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessStatus::Initialized => write!(f, "Initialized"),
            ProcessStatus::Started => write!(f, "Started"),
            ProcessStatus::Starting => write!(f, "Starting"),
            ProcessStatus::Paused => write!(f, "Paused"),
            ProcessStatus::Pausing => write!(f, "Pausing"),
            ProcessStatus::Error => write!(f, "Error"),
            ProcessStatus::NotFound => write!(f, "Not Found"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProcessState {
    pub status: ProcessStatus,
    pub ram_usage: u32,
    pub uid: Option<u32>,
    pub tx: Sender<char>,
    pub should_stop: bool,
}

impl ProcessState {
    pub fn new(
        status: ProcessStatus,
        ram_usage: u32,
        uid: Option<u32>,
        tx: Sender<char>,
        should_stop: bool,
    ) -> Self {
        Self {
            status,
            ram_usage,
            uid,
            tx,
            should_stop,
        }
    }

    pub fn default(tx: Sender<char>) -> Self {
        Self::new(ProcessStatus::Initialized, 0, None, tx, false)
    }
}

#[derive(Clone, Debug)]
pub struct Process {
    pub id: u32,
    pub ecosystem: Ecosystem,
    pub path: String,
    pub state: Arc<Mutex<ProcessState>>,
}

impl Process {
    pub fn new(id: u32, ecosystem: Ecosystem, path: String, process_state: ProcessState) -> Self {
        Self {
            id,
            ecosystem,
            path,
            state: Arc::new(Mutex::new(process_state)),
        }
    }

    pub fn init(id: u32, ecosystem: Ecosystem, path: String) -> Self {
        let (tx, _rx) = broadcast::channel::<char>(1);
        Self::new(id, ecosystem, path, ProcessState::default(tx))
    }

    pub fn from_backup(backup: BackupProcess) -> Self {
        let (tx, _rx) = broadcast::channel::<char>(1);
        Self::new(
            backup.id,
            backup.ecosystem,
            backup.path,
            ProcessState {
                should_stop: backup.should_stop,
                ..ProcessState::default(tx)
            },
        )
    }

    pub async fn spawn(self: Arc<Self>) -> Option<Child> {
        let mut child = Command::new(&self.ecosystem.shell);
        child
            .current_dir(&self.path)
            .arg("-c")
            .arg(&self.ecosystem.start)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        match child.spawn() {
            Ok(child) => Some(child),
            Err(_err) => None,
        }
    }

    pub async fn run(self: Arc<Self>) {
        let process = self;
        if process.state.lock().await.should_stop || process.exists().await {
            return;
        }
        process.state.lock().await.status = ProcessStatus::Starting;
        'main: loop {
            if process.state.lock().await.should_stop || process.exists().await {
                return;
            }
            println!("Starting...");
            let child = process.clone().spawn().await;
            let child = if let Some(child) = child {
                child
            } else {
                break;
            };
            {
                process.state.lock().await.uid = child.id();
            }
            process.state.lock().await.status = ProcessStatus::Started;
            println!("Spawned child: {:?}", child.id());

            // Handle logs
            let child = process.logs(child).await;

            // Handle status
            process.clone().watch_status(child).await;

            // Handle watch
            process.clone().watch_files().await;

            let mut rx = {
                let state = process.state.lock().await;
                state.tx.subscribe()
            };
            while let Ok(data) = rx.recv().await {
                match data {
                    'r' => {
                        if process.exists().await {
                            process.kill().await;
                        }
                        continue 'main;
                    }
                    's' => {
                        if process.exists().await {
                            process.kill().await;
                        }
                        process.state.lock().await.uid = None;
                        break 'main;
                    }
                    _ => {}
                }
            }
        }
    }

    pub async fn watch_files(self: Arc<Self>) {
        let process_tx = self.state.lock().await.tx.clone();
        tokio::spawn(async move {
            let watch = if let Some(watch) = self.ecosystem.watch.clone() {
                watch
            } else {
                return;
            };
            let (tx, mut rx) = mpsc::channel(100);

            let mut watcher = notify::recommended_watcher(move |res: Result<Event, _>| {
                let _ = tx.blocking_send(res);
            })
            .expect("Failed to create watcher");

            for path in watch {
                watcher
                    .watch(
                        &PathBuf::from(&self.path).join(path),
                        RecursiveMode::Recursive,
                    )
                    .expect("Failed to watch");
            }

            while let Some(Ok(event)) = rx.recv().await {
                for path in &event.paths {
                    if let Some(logs) = self.ecosystem.logs.clone() {
                        println!("{:#?}", path);
                        if path.ends_with(logs) {
                            continue;
                        }
                    }
                    process_tx.send('r');
                }
            }
        });
    }

    pub async fn logs(&self, mut child: Child) -> Child {
        let process = self;
        let logs = if let Some(logs) = process.ecosystem.logs.clone() {
            logs
        } else {
            return child;
        };
        let logs = Arc::new(format!("{}/{}", &process.path, logs));
        let stdout = child.stdout.take().expect("Error: Can't handle to stdout");
        let stderr = child.stderr.take().expect("Error: Can't handle to stderr");
        let mut stdout_reader = BufReader::new(stdout).lines();
        let mut stderr_reader = BufReader::new(stderr).lines();

        let log_path = logs.clone();
        match process.ecosystem.logs_mode {
            Some(LogsMode::Append) => {
                let file_content = fs::read_to_string(&*log_path)
                    .await
                    .unwrap_or("".to_string());
                let now: DateTime<Utc> = Utc::now();
                let time_string: String = now.format("%H:%M:%S").to_string();
                let title = format!("=====STARTED ON {}=====", time_string);
                let _ = fs::write(&*log_path, format!("{}\n\n{}\n\n", file_content, title)).await;
            }
            _ => {
                let _ = fs::write(&*log_path, "").await;
            }
        }

        let log_path = logs.clone();
        let _stdout_task = tokio::spawn(async move {
            while let Ok(Some(line)) = stdout_reader.next_line().await {
                println!("{}", line);
                let file_content = fs::read_to_string(&*log_path)
                    .await
                    .unwrap_or("".to_string());
                let _ = fs::write(&*log_path, format!("{}\n{}", file_content, line)).await;
            }
        });
        let log_path = logs.clone();
        let _stderr_task = tokio::spawn(async move {
            while let Ok(Some(line)) = stderr_reader.next_line().await {
                let file_content = fs::read_to_string(&*log_path)
                    .await
                    .unwrap_or("".to_string());
                let _ = fs::write(&*log_path, format!("{}\n{}", file_content, line)).await;
            }
        });

        child
    }

    pub async fn watch_status(self: Arc<Self>, mut child: Child) {
        let process = self;
        let restart_mode = process.ecosystem.restart.unwrap_or(RestartMode::Always);
        let status = child
            .wait()
            .await
            .expect("child process encountered an error");
        let send_code = async |ch: char| {
            process.state.lock().await.tx.send(ch);
        };
        match restart_mode {
            RestartMode::Always => {
                send_code('r').await;
            }
            RestartMode::OnError => {
                if !status.success() {
                    send_code('r').await;
                }
            }
            _ => send_code('s').await,
        }
        println!("child status was: {}", status);
    }

    pub async fn kill(&self) {
        let mut state = self.state.lock().await;

        if let Some(pid) = state.uid {
            let pid = Pid::from_raw(pid as i32);

            if let Err(err) = kill(pid, Signal::SIGTERM) {
                eprintln!("Failed to send SIGTERM: {}", err);
                return;
            } else {
                state.status = ProcessStatus::Paused;
            }

            for _ in 0..30 {
                if kill(pid, None).is_err() {
                    println!("Process gracefully exited");
                    state.status = ProcessStatus::Paused;
                    return;
                }
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }

            println!("Process didn't stop, killing...");
            let _ = kill(pid, Signal::SIGKILL);
            state.status = ProcessStatus::Paused;
        } else {
            state.status = ProcessStatus::Paused;
        }
    }

    pub async fn stop(&self) {
        {
            let mut state = self.state.lock().await;
            state.status = ProcessStatus::Pausing;
            state.should_stop = true;
            state.tx.send('s');
        }
        self.kill().await;
    }

    pub async fn restart(&self) {
        self.kill().await;
        {
            let mut state = self.state.lock().await;
            state.status = ProcessStatus::Pausing;
            state.should_stop = false;
            state.tx.send('r');
        }
    }

    pub async fn exists(&self) -> bool {
        if let Some(uid) = self.state.lock().await.uid {
            Path::new(&format!("/proc/{}", uid)).exists()
        } else {
            false
        }
    }
}

pub async fn start_processes(app_state: Arc<Mutex<crate::AppState>>) {
    let backup = backup::load_backup().await;
    for process in backup {
        let process = Process::from_backup(process);
        let path = PathBuf::from(&process.path);
        let ecosystem_path = path.join(ECOSYSTEM_NAME);
        if ecosystem_path.exists() {
            let id = app_state.lock().await.id;
            app_state.lock().await.id += 1;

            if !path.exists() {
                eprintln!("Invalid path process {id}");
                {
                    let mut process_guard = process.state.lock().await;
                    process_guard.should_stop = true;
                    process_guard.status = ProcessStatus::NotFound;
                }
                continue;
            };
            let process = Arc::new(process);

            app_state.lock().await.processes.insert(id, process.clone());

            tokio::spawn(async move {
                process.run().await;
            });
        }
    }
}
