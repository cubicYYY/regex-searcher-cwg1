type PathVec = Vec<PathBuf>;
type RegexVec = Vec<Regex>;
use colored::*;
use regex::Regex;
use std;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use tracing::{info, instrument};

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
pub fn find(
    path_strs: &Vec<&String>,
    regex_strs: &Vec<&String>,
    is_verbose: bool,
) -> Result<HashSet<String>, Box<dyn std::error::Error>> {
    // Pre-checks
    let mut paths: PathVec = Vec::with_capacity(path_strs.len());
    let mut regexes: RegexVec = Vec::with_capacity(regex_strs.len());

    for path in path_strs {
        if !is_valid_path(path.as_str()) {
            eprintln!("无效的路径'{}'", path);
            process::exit(1);
        }
        paths.push(PathBuf::from(path));
    }

    for regex in regex_strs {
        if !is_valid_pattern(regex) {
            eprintln!("无效的正则表达式'{}'", regex);
            process::exit(1);
        }
        regexes.push(Regex::new(regex).unwrap());
    }
    info!("Search starting...");
    let mut matches = HashSet::new();
    for path in paths {
        walk_tree(path.as_ref(), regexes.as_ref(), &mut matches, 0, is_verbose)?;
    }
    Ok(matches)
}

#[instrument]
fn walk_tree(
    current: &Path,
    regexes: &RegexVec,
    matches: &mut HashSet<String>,
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
                matches.insert(get_full_name(&current));
            } else {
                print!("|{}", String::from("--").repeat(level));
                println!("{}", format!("[-] 检查: {}", filename));
            }
        }
    }
    Ok(())
}
