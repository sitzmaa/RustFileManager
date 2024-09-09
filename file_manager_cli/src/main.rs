use clap::{Arg, Command};
use std::fs;
use std::io::{self, BufRead, Write}; // Import BufRead for read_line method
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use walkdir::WalkDir;
use chrono::{NaiveDateTime, TimeZone};
use glob::glob;

fn main() {
    let matches = Command::new("File Manager CLI")
        .version("1.0")
        .author("Alexander Sitzman")
        .about("A simple CLI tool to manage files")
        .arg(Arg::new("list")
            .short('l')
            .long("list")
            .value_name("DIRECTORY")
            .help("Lists all files in the given directory"))
        .arg(Arg::new("delete")
            .short('d')
            .long("delete")
            .value_name("FILE")
            .help("Deletes the specified file"))
        .arg(Arg::new("rename")
            .short('r')
            .long("rename")
            .value_name("OLD NEW")
            .help("Renames a file from OLD to NEW")
            .number_of_values(2))
        .arg(Arg::new("move")
            .short('m')
            .long("move")
            .value_name("SOURCE DESTINATION")
            .help("Moves a file from SOURCE to DESTINATION")
            .number_of_values(2))
        .arg(Arg::new("copy")
            .short('c')
            .long("copy")
            .value_name("SOURCE DESTINATION")
            .help("Copies a file from SOURCE to DESTINATION")
            .number_of_values(2))
        .arg(Arg::new("mkdir")
            .long("mkdir")
            .value_name("DIRECTORY")
            .help("Creates a new directory"))
        .arg(Arg::new("organize-by-type")
            .long("organize-by-type")
            .value_name("DIRECTORY")
            .help("Organizes files in the directory by type"))
        .arg(Arg::new("organize-by-date")
            .long("organize-by-date")
            .value_name("DIRECTORY")
            .help("Organizes files in the directory by creation or modification date"))
        .arg(Arg::new("find")
            .long("find")
            .value_name("PATTERN")
            .help("Finds files matching the given pattern"))
        .arg(Arg::new("stats")
            .long("stats")
            .value_name("DIRECTORY")
            .help("Displays file statistics (count and total size)"))
        .arg(Arg::new("interactive")
            .long("interactive")
            .help("Starts an interactive CLI mode"))
        .get_matches();

    if matches.get_one::<&str>("interactive").is_some() {
        interactive_mode();
    } else {
        if let Some(dir) = matches.get_one::<&str>("list") {
            list_files(dir);
        }
        if let Some(file) = matches.get_one::<&str>("delete") {
            delete_file(file);
        }
        if let Some(rename_values) = matches.get_many::<String>("rename") {
            let values: Vec<&str> = rename_values.map(|s| s.as_str()).collect();
            rename_file(values[0], values[1]);
        }
        if let Some(move_values) = matches.get_many::<String>("move") {
            let values: Vec<&str> = move_values.map(|s| s.as_str()).collect();
            move_file(values[0], values[1]);
        }
        if let Some(copy_values) = matches.get_many::<String>("copy") {
            let values: Vec<&str> = copy_values.map(|s| s.as_str()).collect();
            copy_file(values[0], values[1]);
        }
        if let Some(dir) = matches.get_one::<&str>("mkdir") {
            create_directory(dir);
        }
        if let Some(dir) = matches.get_one::<&str>("organize-by-type") {
            organize_by_type(dir);
        }
        if let Some(dir) = matches.get_one::<&str>("organize-by-date") {
            organize_by_date(dir);
        }
        if let Some(pattern) = matches.get_one::<&str>("find") {
            find_files(pattern);
        }
        if let Some(dir) = matches.get_one::<&str>("stats") {
            file_stats(dir);
        }
    }
}

// Function to list all files in a directory
fn list_files(dir: &str) {
    println!("Listing files in directory: {}", dir);
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            println!("{}", entry.path().display());
        }
    }
}

// Function to delete a file
fn delete_file(file: &str) {
    match fs::remove_file(file) {
        Ok(_) => println!("File '{}' deleted successfully", file),
        Err(err) => eprintln!("Error deleting file '{}': {}", file, err),
    }
}

// Function to rename a file
fn rename_file(old_name: &str, new_name: &str) {
    match fs::rename(old_name, new_name) {
        Ok(_) => println!("File '{}' renamed to '{}'", old_name, new_name),
        Err(err) => eprintln!("Error renaming file: {}", err),
    }
}

// Function to move a file
fn move_file(source: &str, destination: &str) {
    match fs::rename(source, destination) {
        Ok(_) => println!("File '{}' moved to '{}'", source, destination),
        Err(err) => eprintln!("Error moving file: {}", err),
    }
}

// Function to copy a file
fn copy_file(source: &str, destination: &str) {
    match fs::copy(source, destination) {
        Ok(_) => println!("File '{}' copied to '{}'", source, destination),
        Err(err) => eprintln!("Error copying file: {}", err),
    }
}

// Function to create a new directory
fn create_directory(dir: &str) {
    match fs::create_dir_all(dir) {
        Ok(_) => println!("Directory '{}' created successfully", dir),
        Err(err) => eprintln!("Error creating directory '{}': {}", dir, err),
    }
}

// Function to organize files by type
fn organize_by_type(dir: &str) {
    let dir = Path::new(dir);
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let file_type = entry.path().extension().unwrap_or_default().to_str().unwrap_or("unknown");
            let type_dir = dir.join(file_type);
            if !type_dir.exists() {
                fs::create_dir_all(&type_dir).expect("Failed to create type directory");
            }
            let new_path = type_dir.join(entry.path().file_name().unwrap());
            fs::rename(entry.path(), new_path).expect("Failed to move file");
        }
    }
}

// Function to organize files by date
fn organize_by_date(dir: &str) {
    let dir = Path::new(dir);
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    let datetime = modified.duration_since(UNIX_EPOCH).unwrap();
                    let date = NaiveDateTime::from_timestamp_opt(datetime.as_secs() as i64, 0)
                        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid timestamp")).unwrap();
                    let date_dir = dir.join(date.format("%Y-%m-%d").to_string());
                    if !date_dir.exists() {
                        fs::create_dir_all(&date_dir).expect("Failed to create date directory");
                    }
                    let new_path = date_dir.join(entry.path().file_name().unwrap());
                    fs::rename(entry.path(), new_path).expect("Failed to move file");
                }
            }
        }
    }
}

// Function to find files by pattern
fn find_files(pattern: &str) {
    for entry in glob(pattern).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => println!("{}", path.display()),
            Err(e) => eprintln!("Error reading pattern: {}", e),
        }
    }
}

// Function to display file statistics
fn file_stats(dir: &str) {
    let mut total_size = 0;
    let mut file_count = 0;

    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let metadata = entry.metadata().expect("Failed to get metadata");
            total_size += metadata.len();
            file_count += 1;
        }
    }

    println!("Total files: {}", file_count);
    println!("Total size: {} bytes", total_size);
}

// Function for interactive CLI mode
fn interactive_mode() {
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    println!("Entering interactive mode. Type 'exit' to quit.");
    loop {
        print!("> ");
        io::stdout().flush().expect("Failed to flush stdout");

        let mut input = String::new();
        handle.read_line(&mut input).expect("Failed to read line");

        let input = input.trim();
        if input == "exit" {
            break;
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        match parts.get(0) {
            Some(&"list") => {
                if let Some(dir) = parts.get(1) {
                    list_files(dir);
                } else {
                    println!("Usage: list <directory>");
                }
            }
            Some(&"delete") => {
                if let Some(file) = parts.get(1) {
                    delete_file(file);
                } else {
                    println!("Usage: delete <file>");
                }
            }
            Some(&"rename") => {
                if let (Some(old_name), Some(new_name)) = (parts.get(1), parts.get(2)) {
                    rename_file(old_name, new_name);
                } else {
                    println!("Usage: rename <old_name> <new_name>");
                }
            }
            Some(&"move") => {
                if let (Some(source), Some(destination)) = (parts.get(1), parts.get(2)) {
                    move_file(source, destination);
                } else {
                    println!("Usage: move <source> <destination>");
                }
            }
            Some(&"copy") => {
                if let (Some(source), Some(destination)) = (parts.get(1), parts.get(2)) {
                    copy_file(source, destination);
                } else {
                    println!("Usage: copy <source> <destination>");
                }
            }
            Some(&"mkdir") => {
                if let Some(dir) = parts.get(1) {
                    create_directory(dir);
                } else {
                    println!("Usage: mkdir <directory>");
                }
            }
            Some(&"organize-by-type") => {
                if let Some(dir) = parts.get(1) {
                    organize_by_type(dir);
                } else {
                    println!("Usage: organize-by-type <directory>");
                }
            }
            Some(&"organize-by-date") => {
                if let Some(dir) = parts.get(1) {
                    organize_by_date(dir);
                } else {
                    println!("Usage: organize-by-date <directory>");
                }
            }
            Some(&"find") => {
                if let Some(pattern) = parts.get(1) {
                    find_files(pattern);
                } else {
                    println!("Usage: find <pattern>");
                }
            }
            Some(&"stats") => {
                if let Some(dir) = parts.get(1) {
                    file_stats(dir);
                } else {
                    println!("Usage: stats <directory>");
                }
            }
            _ => println!("Unknown command: {}", parts[0]),
        }
    }
}
