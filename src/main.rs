pub mod regex_searcher;

use clap::{App, Arg};
use colored::*;
use regex_searcher::find;
use std::process;
use tracing::info;
use tracing_subscriber::FmtSubscriber;

fn main() {
    // Init info tracker (from tracing)
    let subscriber = FmtSubscriber::builder().finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");

    // Set all args
    // I should use a declarative way...
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

    info!(
        "{} 条路径",
        matches.get_many::<String>("paths").unwrap().count()
    );
    info!(
        "{} 条正则规则",
        matches.get_many::<String>("regexes").unwrap().count()
    );

    // Ready to search!
    match find(
        &matches.get_many::<String>("paths").unwrap().collect(),
        &matches.get_many::<String>("regexes").unwrap().collect(),
        matches.is_present("verbose"),
    ) {
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
