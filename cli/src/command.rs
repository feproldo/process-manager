use std::process;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};

use crate::{answer::Answer, SOCKET_PATH};

pub enum Commands {
    Start(String),
    // Remove(String),
    Stop(String),
    Status(Option<String>),
    Backup,
    Restart(String),
    Delete(String),
    Load,
}

impl Commands {
    pub async fn send(&self) -> Answer {
        let (command, message): (&str, String) = match self {
            Self::Start(message) => ("start", message.clone()),
            Self::Restart(message) => ("restart", message.clone()),
            Self::Stop(message) => ("stop", message.clone()),
            Self::Status(message) => ("status", message.clone().unwrap_or("".to_string())),
            Self::Backup => ("backup", "".to_string()),
            Self::Delete(message) => ("delete", message.clone()),
            Self::Load => ("message", "".to_string()),
        };
        let mut stream = if let Ok(stream) = UnixStream::connect(SOCKET_PATH).await {
            stream
        } else {
            eprintln!("Error: Can't connect UnixStream");
            process::exit(1);
        };
        stream
            .write_all(&format!("{} {}", command, message).as_bytes())
            .await
            .unwrap();
        let mut buf = [0; 1024];
        let n = stream.read(&mut buf).await.unwrap();
        String::from_utf8_lossy(&buf[..n]).into()
    }
}
