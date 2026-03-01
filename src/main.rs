mod config;
mod highlight;
mod jemdoc;
mod text;

use std::fs;
use std::io::{self, BufWriter, Write};
use std::path::Path;
use std::process;

use config::{parse_conf, show_config};
use jemdoc::JemdocParser;

const VERSION: &str = "jemdoc-rs version 0.7.3 (Rust rewrite)";

fn show_help() {
    let help = r#"Usage: jemdoc-rs [OPTIONS] [SOURCEFILE]
Produces html markup from a jemdoc SOURCEFILE.

Most of the time you can use jemdoc-rs without any additional flags.
For example, typing

  jemdoc-rs index

will produce an index.html from index.jemdoc, using a default
configuration.

Some configuration options can be overridden by specifying a
configuration file.  You can use

  jemdoc-rs --show-config

to print a sample configuration file (which includes all of the
default options). Any or all of the configuration [blocks] can be
overwritten by including them in a configuration file, and running,
for example,

  jemdoc-rs -c mywebsite.conf index.jemdoc

You can view version and installation details with

  jemdoc-rs --version

See https://github.com/haozhu10015/jemdoc-rs for more details."#;
    println!("{}", help);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 1 || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string())
    {
        show_help();
        process::exit(0);
    }

    if args.contains(&"--show-config".to_string()) {
        show_config();
        process::exit(0);
    }

    if args.contains(&"--version".to_string()) {
        println!("{}", VERSION);
        process::exit(0);
    }

    let mut outname: Option<String> = None;
    let mut confnames: Vec<String> = Vec::new();
    let mut i = 1;

    // Parse flags
    while i < args.len() {
        if args[i] == "-o" {
            if outname.is_some() {
                eprintln!("Error: only one output file / directory, please");
                process::exit(1);
            }
            i += 1;
            if i >= args.len() {
                eprintln!("Error: -o requires an argument");
                process::exit(1);
            }
            outname = Some(args[i].clone());
            i += 1;
        } else if args[i] == "-c" {
            i += 1;
            if i >= args.len() {
                eprintln!("Error: -c requires an argument");
                process::exit(1);
            }
            confnames.push(args[i].clone());
            i += 1;
        } else if args[i].starts_with('-') {
            eprintln!("Error: unrecognised argument {}, try --help", args[i]);
            process::exit(1);
        } else {
            break;
        }
    }

    // Parse configuration
    let conf = parse_conf(&confnames);

    // Collect input filenames
    let mut innames: Vec<String> = Vec::new();
    while i < args.len() {
        let mut inname = args[i].clone();
        // If not a file and no dot, try appending .jemdoc
        if !Path::new(&inname).is_file() && !inname.contains('.') {
            inname.push_str(".jemdoc");
        }
        innames.push(inname);
        i += 1;
    }

    if innames.is_empty() {
        eprintln!("Error: no input files specified");
        process::exit(1);
    }

    if let Some(ref oname) = outname {
        if !Path::new(oname).is_dir() && innames.len() > 1 {
            eprintln!("Error: cannot handle one outfile with multiple infiles");
            process::exit(1);
        }
    }

    // Process each input file
    for inname in &innames {
        let thisout = if let Some(ref oname) = outname {
            if Path::new(oname).is_dir() {
                let base = inname.trim_end_matches(".jemdoc");
                format!("{}{}.html", oname, base)
            } else {
                oname.clone()
            }
        } else {
            let base = inname.trim_end_matches(".jemdoc");
            format!("{}.html", base)
        };

        // Read input file
        let content = match fs::read_to_string(inname) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error reading {}: {}", inname, e);
                process::exit(1);
            }
        };

        let lines: Vec<String> = content.lines().map(|l| format!("{}\n", l)).collect();

        // Open output file
        let outfile: Box<dyn Write> = if thisout == "-" {
            Box::new(BufWriter::new(io::stdout()))
        } else {
            match fs::File::create(&thisout) {
                Ok(f) => Box::new(BufWriter::new(f)),
                Err(e) => {
                    eprintln!("Error creating {}: {}", thisout, e);
                    process::exit(1);
                }
            }
        };

        let mut parser = JemdocParser::new(inname.clone(), lines, outfile, conf.clone());
        parser.proc_file();

        eprintln!("{} -> {}", inname, thisout);
    }
}