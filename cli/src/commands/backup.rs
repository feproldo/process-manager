use crate::{command::Commands, commands::Arguments};

pub async fn exec(args: Arguments) {
    Commands::Backup.send().await;
}
