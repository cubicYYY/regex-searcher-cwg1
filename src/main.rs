use clap::{App, Arg};
use colored::*;
use regex::Regex;
use std::path::Path;
use std::process;
use std::{fs, path::PathBuf};
use tracing::{info, instrument};
use tracing_subscriber::FmtSubscriber;

type PathVec = Vec<PathBuf>;
type RegexVec = Vec<Regex>;

fn main() {
    let subscriber = FmtSubscriber::builder().finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");

    let matches = App::new("Regex")
        .arg(
            Arg::new("paths")
                .short('p')
                .long("paths")
                .help("List of paths to search")
                .takes_value(true)
                .multiple_values(true)
                .required(true),
        )
        .arg(
            Arg::new("regexes")
                .short('r')
                .long("regexes")
                .help("List of regex expressions to match, all results combined.")
                .takes_value(true)
                .multiple_values(true)
                .required(true),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Show searching detailed infos")
                .takes_value(false)
                .required(false),
        )
        .get_matches();

    // Pre-checks
    let mut paths: PathVec =
        Vec::with_capacity(matches.get_many::<String>("paths").unwrap().count());
    let mut regexes: RegexVec =
        Vec::with_capacity(matches.get_many::<String>("regexes").unwrap().count());
    info!(
        "{} 条路径",
        matches.get_many::<String>("paths").unwrap().count()
    );
    info!(
        "{} 条正则规则",
        matches.get_many::<String>("regexes").unwrap().count()
    );

    if let Some(path_strs) = matches.get_many::<String>("paths") {
        for path in path_strs {
            if !is_valid_path(path.as_str()) {
                eprintln!("无效的路径'{}'", path);
                process::exit(1);
            }
            paths.push(PathBuf::from(path));
        }
    }

    if let Some(regex_strs) = matches.get_many::<String>("regexes") {
        for regex in regex_strs {
            if !is_valid_pattern(regex) {
                eprintln!("无效的正则表达式'{}'", regex);
                process::exit(1);
            }
            regexes.push(Regex::new(regex).unwrap());
        }
    }

    // Ready to search
    match find(&paths, &regexes, matches.is_present("verbose")) {
        Ok(matches) => {
            println!("{}", "=".repeat(30));
            if matches.is_empty() {
                println!("{}", "未找到匹配项。".red());
            } else {
                println!("{}", "找到以下匹配项:".green());
                for file in matches {
                    println!("{}", file);
                }
            }
            println!("{}", "=".repeat(30));
        }
        Err(error) => {
            eprintln!("发生错误:{}", error);
            process::exit(1);
        }
    }
}

fn is_valid_path(path_str: &str) -> bool {
    Path::new(path_str).exists()
}

fn is_valid_pattern(pattern: &str) -> bool {
    match Regex::new(pattern) {
        Ok(_) => true,
        Err(_) => false,
    }
}

#[instrument] // to log internally
fn find(
    paths: &PathVec,
    regexes: &RegexVec,
    is_verbose: bool,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    info!("Search starting...");
    let mut matches = Vec::new();
    for path in paths {
        walk_tree(path.as_ref(), regexes, &mut matches, 0, is_verbose)?;
    }
    Ok(matches)
}

#[instrument]
fn walk_tree(
    current: &Path,
    regexes: &RegexVec,
    matches: &mut Vec<String>,
    level: usize,
    is_verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let get_full_name: fn(&Path) -> String = |x: &Path| x.to_string_lossy().to_string();
    if current.is_dir() {
        if is_verbose {
            print!("|{}", String::from("--").repeat(level));
            println!("搜寻目录: {}", get_full_name(&current));
        }
        for entry in fs::read_dir(current)? {
            let path = entry?.path();
            walk_tree(&path, regexes, matches, level + 1, is_verbose)?;
        }
    } else if let Some(filename) = current.file_name().and_then(|s| s.to_str()) {
        for regex in regexes {
            if regex.is_match(filename) {
                if is_verbose {
                    print!("|{}", String::from("--").repeat(level));
                    println!("{}", format!("[*] 匹配成功: {}", filename).green());
                }
                matches.push(get_full_name(&current));
            } else {
                print!("|{}", String::from("--").repeat(level));
                println!("{}", format!("[-] 检查: {}", filename));
            }
        }
    }
    Ok(())
}
