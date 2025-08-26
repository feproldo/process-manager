use std::{borrow::Cow, fmt::Display, path::Path, process};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};

use crate::{answer::Answer, SOCKET_PATH};

pub mod backup;
pub mod delete;
pub mod ecosystem;
pub mod help;
pub mod restart;
pub mod start;
pub mod status;
pub mod stop;

pub struct Arguments {
    pub positional: Vec<String>,
    pub flags: Vec<char>,
    pub options: Vec<String>,
}

impl Arguments {
    pub fn default() -> Self {
        Self {
            positional: vec![],
            flags: vec![],
            options: vec![],
        }
    }
}

pub async fn handle_command(args: Vec<String>) {
    let mut args = args;
    if args.len() <= 0 {
        args.push("help".to_string());
    }
    let command = args.remove(0);
    let mut arguments = Arguments::default();
    for arg in args {
        if arg.starts_with("-") {
            if arg.starts_with("--") {
                let mut arg = arg;
                arg.remove(0);
                arg.remove(0);
                arguments.options.push(arg);
            } else {
                let mut arg = arg;
                arg.remove(0);
                for ch in arg.chars() {
                    arguments.flags.push(ch);
                }
            }
        } else {
            arguments.positional.push(arg);
        }
    }
    match command.as_str() {
        "ecosystem" => ecosystem::exec(arguments).await,
        "start" | "letsgo" => start::exec(arguments).await,
        "stop" | "pause" => stop::exec(arguments).await,
        "backup" | "save" => backup::exec(arguments).await,
        "restart" | "reload" => restart::exec(arguments).await,
        "status" => status::exec(arguments).await,
        "delete" => delete::exec(arguments).await,
        "help" => help::exec(),
        _ => help::exec(),
    }
}
