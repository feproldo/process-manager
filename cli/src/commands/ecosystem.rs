use std::{fmt::Display, io::stdin, process};

use dialoguer::{console::Color, theme::ColorfulTheme, Input, Select};
use tokio::fs;

#[derive(Clone, Debug)]
pub struct Ecosystem {
    pub title: String,
    pub start: String,
    pub shell: String,
    pub description: Option<String>,
    pub restart: Option<String>,
    pub logs: Option<String>,
    pub logs_mode: Option<String>,
    pub watch: Option<Vec<String>>,
}

impl Default for Ecosystem {
    fn default() -> Self {
        Self {
            title: String::from(""),
            start: String::from(""),
            shell: String::from(""),
            description: None,
            restart: None,
            logs: None,
            logs_mode: None,
            watch: None,
        }
    }
}

impl Display for Ecosystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const QUOTES: &str = "\"";
        const QUOTES_REPLACE: &str = "\\\"";
        let mut result = format!(
            "title=\"{}\"\nstart=\"{}\"\nshell=\"{}\"",
            self.title.replace(QUOTES, QUOTES_REPLACE),
            self.start.replace(QUOTES, QUOTES_REPLACE),
            self.shell.replace(QUOTES, QUOTES_REPLACE)
        );
        if let Some(description) = self.description.clone() {
            result = format!(
                "{result}\ndescription=\"{}\"",
                description.replace(QUOTES, QUOTES_REPLACE)
            );
        };
        if let Some(restart) = self.restart.clone() {
            result = format!(
                "{result}\nrestart=\"{}\"",
                restart.replace(QUOTES, QUOTES_REPLACE)
            );
        };
        if let Some(logs) = self.logs.clone() {
            result = format!(
                "{result}\nlogs=\"{}\"",
                logs.replace(QUOTES, QUOTES_REPLACE)
            );
        };
        if let Some(logs_mode) = self.logs_mode.clone() {
            result = format!(
                "{result}\nlogs_mode=\"{}\"",
                logs_mode.replace(QUOTES, QUOTES_REPLACE)
            );
        };
        if let Some(watch) = self.watch.clone() {
            result = format!("{result}\nwatch=[",);
            for (index, el) in watch.iter().enumerate() {
                if index == 0 {
                    result = format!("{result}\"{}\"", el.replace(QUOTES, QUOTES_REPLACE));
                } else {
                    result = format!("{result}, \"{}\"", el.replace(QUOTES, QUOTES_REPLACE));
                }
            }
            result = format!("{result}]")
        };
        write!(f, "{result}")
    }
}

use crate::{commands::Arguments, DOCUMENTATION_URL, ECOSYSTEM_NAME};

pub async fn exec(args: Arguments) {
    if args.flags.contains(&'q') {
        let default_ecosystem = Ecosystem {
            title: "your project".to_string(),
            start: "echo \"hello shell!\" && ./your_app".to_string(),
            shell: "/bin/sh".to_string(),
            description: Some("Your cool project to launch in fpm".to_string()),
            restart: Some("always".to_string()),
            logs: Some("fpm-log.txt".to_string()),
            logs_mode: Some("override".to_string()),
            watch: Some(vec![".".to_string()]),
        };
        if let Err(err) = fs::write(
            ECOSYSTEM_NAME,
            format!(
                "# Documentation: {}\n[[process]]\n{}",
                DOCUMENTATION_URL, default_ecosystem
            ),
        )
        .await
        {
            eprintln!("Error. Can't create file: {}", err);
            process::exit(1);
        };
        return ();
    }
    setup_file().await;
}

async fn setup_file() {
    let mut ecosystem = Ecosystem::default();
    let title: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Name to display?")
        .interact_text()
        .unwrap();
    ecosystem.title = title;
    let command: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Command to start?")
        .interact_text()
        .unwrap();
    ecosystem.start = command;
    let shell: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Shell path?")
        .interact_text()
        .unwrap();
    ecosystem.shell = shell;
    let options = vec!["always", "never", "on error"];
    let restart = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("When restart?")
        .items(&options)
        .default(0)
        .interact()
        .unwrap();

    ecosystem.restart = Some(
        match restart {
            1 => "never",
            2 => "on_error",
            _ => "always",
        }
        .to_string(),
    );

    let options = vec!["yes", "no"];
    let watch = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Watch files?")
        .items(&options)
        .default(1)
        .interact()
        .unwrap();
    if watch == 0 {
        let watch: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt(
                    "Which files need wathing? (use releative path to files/directoies and ',' to separate. Example: \"src/main.rs,src/pages/index.rs\")",
                )
                .interact_text()
                .unwrap();
        ecosystem.watch = Some(watch.split(",").map(|file| file.to_string()).collect());
    }
    let log = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Log stdout?")
        .items(&options)
        .default(1)
        .interact()
        .unwrap();
    if log == 0 {
        let log_file: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Which file should the logs be written to?")
            .interact_text()
            .unwrap();
        ecosystem.logs = Some(log_file);
        let options = vec!["override", "append"];
        let log_mode = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Log mode?")
            .items(&options)
            .default(0)
            .interact()
            .unwrap();
        if log_mode == 0 {
            ecosystem.logs_mode = Some("override".to_string());
        } else {
            ecosystem.logs_mode = Some("append".to_string());
        };
    }
    if let Err(err) = fs::write(ECOSYSTEM_NAME, format!("{}", ecosystem)).await {
        eprintln!("{err}");
        process::exit(1);
    }
}
