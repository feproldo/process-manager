use std::{borrow::Cow, error::Error, fs, path::Path, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{UnixListener, UnixStream},
    sync::Mutex,
};

use crate::commands;

const SOCKET_PATH: &str = "/tmp/fpm.sock";

pub enum Response {
    Successfully(Option<&'static str>),
    Data(String),
    Error(&'static str),
}

impl Response {
    pub fn to_string(self) -> String {
        match self {
            Self::Data(data) => {
                format!("data {}", data)
            }
            Self::Error(err_code) => {
                format!("error {}", err_code.to_string())
            }
            Self::Successfully(message) => format!("success {}", message.unwrap_or("")),
        }
    }
}

pub async fn start_socket(app_state: Arc<Mutex<crate::AppState>>) -> Result<(), Box<dyn Error>> {
    if Path::new(SOCKET_PATH).exists() {
        fs::remove_file(SOCKET_PATH)?;
    }

    let listener = UnixListener::bind(SOCKET_PATH)?;
    println!("Демон слушает на {}...", SOCKET_PATH);

    loop {
        let app_state_clone = app_state.clone();
        let (stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            let _ = handle_connection(app_state_clone, stream).await;
        });
    }
}

async fn handle_connection(
    app_state: Arc<Mutex<crate::AppState>>,
    mut stream: UnixStream,
) -> Result<(), Box<dyn Error>> {
    let mut buf = [0; 1024];

    match stream.read(&mut buf).await {
        Ok(n) if n > 0 => {
            let msg = String::from_utf8_lossy(&buf[..n]);
            println!("Get: {}", msg);
            let answer: Response = match_command(app_state, msg.to_string()).await;
            if let Err(e) = stream
                .write_all(format!("{}\n", answer.to_string()).as_bytes())
                .await
            {
                eprintln!("Error sending: {}", e);
            }
        }
        Ok(_) => println!("Empty"),
        Err(e) => eprintln!("Error reading: {}", e),
    }
    Ok(())
}

async fn match_command(app_state: Arc<Mutex<crate::AppState>>, command: String) -> Response {
    let mut command: Vec<&str> = command.split_whitespace().collect();

    if command.is_empty() {
        return Response::Error("The command was not found");
    }

    let command_name = command.remove(0);
    let arg = command.join(" ");

    let result: Result<Response, Box<dyn Error>> = match command_name {
        "start" => commands::start(app_state, arg).await,
        "stop" => commands::stop(app_state, arg).await,
        "restart" => commands::restart(app_state, arg).await,
        "status" => commands::status(app_state, arg).await,
        "backup" => commands::backup(app_state, arg).await,
        "delete" => commands::delete(app_state, arg).await,
        _ => Ok(Response::Error("Unknown command")),
    };

    if let Ok(answer) = result {
        answer
    } else {
        Response::Error("The internal error of the demon. Please create an issue on github")
    }
}
