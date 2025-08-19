use std::{env, process};
mod answer;
mod command;
mod commands;
mod config;
mod demon;

const ECOSYSTEM_NAME: &str = "pm-ecosystem.toml";
const DOCUMENTATION_URL: &str = "https://fpm.feproldo.ru/";
const SOCKET_PATH: &str = "/tmp/fpm.sock";

#[tokio::main]
async fn main() {
    let _ = config::get_cfg();
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);
    commands::handle_command(args).await;
}
