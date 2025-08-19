use std::process;

use crate::{command::Commands, commands::Arguments, ECOSYSTEM_NAME};

pub async fn exec(args: Arguments) {
    if args.positional.len() > 0 {
        if let Some(arg) = args.positional.get(0) {
            let answer = Commands::Restart(arg.clone()).send().await;
            println!("{}", answer);
            return;
        } else {
            println!("Error: Arguments are not provided");
        }
    }
}
