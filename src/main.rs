extern crate argparse;
extern crate term;
extern crate rust_util;

use argparse::{ArgumentParser, StoreTrue, Store};

use rust_util::*;

use std::{
    fs::File,
    path::Path,
    io::prelude::*,
};


struct Options {
    version: bool,
    verbose: bool,
    dry_run: bool,
    search_text: String,
    replace_text: String,
}

impl Options {
    fn new() -> Options {
        Options {
            version: false,
            verbose: false,
            dry_run: false,
            search_text: String::new(),
            replace_text: String::new(),
        }
    }

    fn new_and_parse_args() -> Options {
        let mut options = Options::new();
        {
            let mut ap = ArgumentParser::new();
            ap.set_description("replacing - command line replace tool.");
            ap.refer(&mut options.version).add_option(&["-v", "--version"], StoreTrue, "Print version");
            ap.refer(&mut options.verbose).add_option(&["--verbose"], StoreTrue, "Verbose");
            ap.refer(&mut options.dry_run).add_option(&["--dry-run"], StoreTrue, "Dry run");
            ap.refer(&mut options.search_text).add_argument("SEARCH TEXT", Store, "Search text");
            ap.refer(&mut options.replace_text).add_argument("REPLACE TEXT", Store, "Replace text");
            ap.parse_args_or_exit();
        }
        options
    }
}

pub fn read_file_content(file: &Path, large_file_len: u64) -> XResult<String> {
    if ! file.exists() {
        return Err(new_box_error(&format!("File not exists: {:?}", file)));
    }
    if ! file.is_file() {
        return Err(new_box_error(&format!("File is not file: {:?}", file)));
    }
    let file_len = file.metadata()?.len();
    if file_len > large_file_len {
        return Err(new_box_error(&format!("File too large: {:?}, len: {}", file, file_len)));
    }
    let mut f = File::open(file)?;
    let mut content = String::new();
    f.read_to_string(&mut content)?;

    Ok(content)
}

pub fn write_file_content(file: &Path, content: &str) -> XResult<()> {
    let mut f = File::create(file)?;
    f.write_all(content.as_ref())?;
    f.sync_all()?;

    Ok(())
}


fn replace_files(options: &Options, dir_path: &Path) -> XResult<()> {
    walk_dir(&dir_path, &|_, _| (/* do not process error */), &|p| {
        let p_str = match p.to_str() {
            None => return,
            Some(s) => s,
        };
        /*
        if file_exts.len() > 0 {
            let mut file_ext_matches = false;
            for i in 0..file_exts.len() {
                if p_str.to_lowercase().ends_with(&file_exts[i]) {
                    file_ext_matches = true;
                    break;
                }
            }
            if ! file_ext_matches {
                return;
            }
        }
        if options.filter_file_name.len() > 0 {
            if ! p_str.contains(options.filter_file_name.as_str()) {
                return;
            }
        }
        */
        let file_content = match read_file_content(p, 100000u64) {
            Err(err) => {
                if options.verbose {
                    print_lastline("");
                    print_message(MessageType::WARN, &format!("Read file {} failed: {}", p_str, err));
                }
                return;
            },
            Ok(c) => c,
        };
        if file_content.contains(&options.search_text) {
            // FOUND
            if options.dry_run {
                clear_lastline();
                print_message(MessageType::OK, &format!("Dry run: {}", p_str));
                return;
            }

            clear_lastline();
            print_message(MessageType::OK, &format!("Write file: {}", p_str));
            let replaced_file_content = file_content.replace(&options.search_text, &options.replace_text);
            println!("{}", file_content);
            println!("{}", replaced_file_content);
            write_file_content(p, &replaced_file_content).ok();
        }
    }, &|p| {
        match p.to_str() {
            None => (),
            Some(p_str) => {
                //if (! options.scan_dot_git) && p_str.ends_with("/.git") {
                //    if options.verbose {
                //       print_lastline("");
                //        print_message(MessageType::INFO, &format!("Skip .git dir: {}", p_str));
                //    }
                //    return false;
                //}
                //if options.skip_dot_dir && p_str.contains("/.") {
                //    return false;
                //}
                //if options.skip_link_dir && is_symlink(p) {
                //    if options.verbose {
                //        print_lastline("");
                //        print_message(MessageType::INFO, &format!("Skip link dir: {}", p_str));
                //    }
                //    return false;
                //}
                print_lastline(&format!("Scanning: {}", p_str));
            },
        }
        true
    }).unwrap_or(());
    clear_lastline();
    Ok(())
}

fn main() -> XResult<()> {
    let options = Options::new_and_parse_args();

    let dir = ".";
    let dir_path = match get_absolute_path(dir) {
        None => {
            return Err(new_box_error(&format!("Cannot find dir: {}", dir)));
        },
        Some(path) => path,
    };

    if options.search_text.len() == 0 || options.replace_text.len() == 0 {
        return Err(new_box_ioerror("Search text and replace text cannot be empty."));
    }

    replace_files(&options, &dir_path)?;

    Ok(())
}
