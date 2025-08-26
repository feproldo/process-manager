pub fn exec() {
    println!("feproldo's process manager");
    println!("Usage:");
    println!("\tfpm <command> [...]");
    println!("");
    println!("Commands:");
    println!("\thelp - shows this text");
    println!("\tstart [id|title] - starts a new process or resumes a stopped one");
    println!("\tstop <id|title> - stops the process");
    println!("\tstatus - displays information about running processes");
    println!("\trestart <id|title> - reads the ecosystem and restarts the process with it");
    println!("\tecosystem [-q] - creates an ecosystem file (configuration file)");
    println!("\tbackup - saves current processes for restarting them in the future");
    println!("\tdelete - stops and removes the process from the list");
}

