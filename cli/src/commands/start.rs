use crate::{command::Commands, commands::Arguments, ECOSYSTEM_NAME};
use std::process;

pub async fn exec(args: Arguments) {
    if args.positional.len() > 0 {
        if let Some(arg) = args.positional.get(0) {
            let answer = Commands::Start(arg.clone()).send().await;
            println!("{}", answer);
            return;
        }
    }
    let current_dir = std::env::current_dir().expect("Can't get current dir.");
    if !current_dir.join(ECOSYSTEM_NAME).exists() {
        eprintln!("Error: Ecosystem file not found in current directory. Use \"fpm ecosystem\"");
        process::exit(1);
    }
    let answer = Commands::Start(current_dir.to_string_lossy().to_string())
        .send()
        .await;
    println!("{}", answer);
}
