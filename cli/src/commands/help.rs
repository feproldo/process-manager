pub fn exec(args: &Vec<String>) {
    let name = &args.get(0).map_or("pm".to_string(), |s| s.to_string()).to_owned()[..];
}