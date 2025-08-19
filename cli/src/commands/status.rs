use crate::{command::Commands, commands::Arguments, ECOSYSTEM_NAME};
use std::process;

pub async fn exec(args: Arguments) {
    if args.positional.len() > 0 {
        if let Some(arg) = args.positional.get(0) {
            let answer = Commands::Status(Some(arg.clone())).send().await;
            println!("{}", answer);
            return;
        }
    }
    let answer = Commands::Status(None).send().await;
    println!("{}", answer);
}
