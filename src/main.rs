use regex::Regex;
use std::env;
use std::fs;
use std::path::Path;
use std::process;
use tracing::{info, instrument};
use tracing_subscriber::FmtSubscriber;
use colored::*;
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("使用方式: {} <目标目录> <要搜索的正则表达式>", args[0]);
        process::exit(1);
    }
    let pattern = &args[2];
    let regex = match Regex::new(pattern) {
        Ok(re) => re,
        Err(err) => {
            eprintln!("无效的正则表达式'{}': {}", pattern, err);
            process::exit(1);
        }
    };
    match find(&args[1], &regex, true) {
        Ok(matches) => {
            if matches.is_empty() {
                println!("未找到匹配项。");
            } else {
                println!("找到以下匹配项:");
                for file in matches {
                    println!("{}", file);
                }
            }
        }
        Err(error) => {
            eprintln!("发生错误:{}", error);
            process::exit(1);
        }
    }
}

fn find<P: AsRef<Path>>(root: P, regex: &Regex, is_verbose: bool) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let subscriber = FmtSubscriber::builder().finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");
    info!("Search starting...");
    let mut matches = Vec::new();
    walk_tree(root.as_ref(), regex, &mut matches, 0, is_verbose)?;
    Ok(matches)
}

#[instrument] // to log internally
fn walk_tree(
    dir: &Path,
    regex: &Regex,
    matches: &mut Vec<String>,
    level: usize,
    is_verbose: bool
) -> Result<(), Box<dyn std::error::Error>> {
    let get_full_name: fn(&Path) -> String = |x: &Path| x.to_string_lossy().to_string();

    if dir.is_dir() {
        if is_verbose {
            print!("{}", String::from("--").repeat(level));
            println!("搜寻子目录: {}", get_full_name(&dir));
        }
        for entry in fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_dir() {
                walk_tree(&path, regex, matches, level + 1, is_verbose)?;
            } else if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                if regex.is_match(filename) {
                    if is_verbose {
                        print!("{}", String::from("--").repeat(level));
                        println!("{}", format!("[*] 匹配成功: {}", filename).green());
                    }
                    matches.push(get_full_name(&path));
                }
            }
        }
    } else {
        eprintln!("提供的路径 {} 非法。", get_full_name(&dir));
        eprintln!("可能原因：该路径非法，该路径指向文件（包括指向文件夹的软链接文件），或对该路径访问权限不足。");
        process::exit(1);
    }
    Ok(())
}
