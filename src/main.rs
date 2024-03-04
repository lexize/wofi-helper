use std::{collections::{HashMap, LinkedList}, env, fs::File, io::{stdout, BufRead, BufReader, Write}, path::Path, process::{exit, Command, Stdio}, thread};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_help();
        return;
    }
    let cfg_separator = env::var("WOFI_CFG_SEPARATOR").unwrap_or(String::from("="));
    let separator = env::var("WOFI_SEPARATOR").unwrap_or(String::from("\n"));
    let commands = parse_conf(args[1].clone(), cfg_separator);
    let mut commands_map = HashMap::new();
    let mut titles: Vec<String> = Vec::new();
    for k in commands {
        titles.push(k.0.clone());
        commands_map.insert(k.0, k.1);
    }
    let input_to_wofi = titles.join(&separator);
    println!("{}", input_to_wofi);
    let mut wofi = 
    Command::new("wofi").args(args[2..].into_iter())
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()
    .expect("Failed to start wofi");
    let mut stdin = wofi.stdin.take().expect("Unable to take stdin");
    thread::spawn(move || {
        let _ = stdin.write_all(input_to_wofi.as_bytes());
    });

    let output = wofi.wait_with_output().expect("Failed to read stdout");
    let out = String::from_utf8(output.stdout).expect("Error occured while reading stdout");
    let stripped_out = String::from(out.strip_suffix("\n").unwrap());
    if let Some(s) = commands_map.get(&stripped_out) {
        let command = s.clone();
        let sh = Command::new("sh")
        .env("WOFI_OUTPUT", &stripped_out)
        .stdin(Stdio::piped())
        .stdout(stdout()).spawn().expect("Failed to start sh");
        
        let mut stdin = sh.stdin.unwrap();
        let _ = stdin.write_all(command.as_bytes());
    }
}

fn parse_conf<P: AsRef<Path>>(path: P, separator: String) -> LinkedList<(String, String)> {
    let f = File::open(path);
    if f.is_err() {
        error("Input file not found");
    }
    let file = f.unwrap();
    let mut reader = BufReader::new(file);
    let mut line = String::new();
    let mut list: LinkedList<(String, String)> = LinkedList::new();
    while let Ok(s) = reader.read_line(&mut line) {
        if s == 0 {
            break;
        }
        let s = line.strip_suffix("\n").map(String::from);
        if let Some(str) = s {
            line.clear();
            line.push_str(str.as_str());
        }
        let split = line.split_once(&separator);
        if let Some((t, c)) = split {
            list.push_back((String::from(t), String::from(c)));
        }
        line.clear();
    }
    return list;
}

fn error(s: &str) {
    println!("{}", s);
    exit(-1);
}

fn print_help() {
    println!("wofi-helper <input-file> [arguments]
    <input-file> must be provided
    [arguments] will be passed to wofi")
}