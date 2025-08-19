use crate::{command::Commands, commands::Arguments, ECOSYSTEM_NAME};
use std::process;

pub async fn exec(args: Arguments) {
    if args.positional.len() < 1 {
        println!("Bad usage: fpm stop <index / title>");
        return;
    }
    if let Some(arg) = args.positional.get(0) {
        println!("{}", Commands::Stop(String::from(arg)).send().await)
    }
}
