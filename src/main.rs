extern crate argparse;
extern crate term;
extern crate rust_util;

mod opt;

use rust_util::*;
use opt::*;

use std::{
    fs::File,
    path::Path,
    io::prelude::*,
};


pub fn get_term_width() -> Option<usize> {
    match term_size::dimensions() {
        None => None,
        Some((w, _h)) => Some(w),
    }
}

// thanks https://blog.csdn.net/star_xiong/article/details/89401149
pub fn find_char_boundary(s: &str, index: usize) -> usize {
    if s.len() <= index {
        return index;
    }
    let mut new_index = index;
    while !s.is_char_boundary(new_index) {
        new_index += 1;
    }
    new_index
}

pub fn get_term_width_message(message: &str, left: usize) -> String {
    match get_term_width() {
        None => message.to_string(),
        Some(w) => {
            let len = message.len();
            if w > len {
               return message.to_string();
            }
            let mut s = String::new();
            s.push_str(&message[0..find_char_boundary(&message, w-10-5-left)]);
            s.push_str("[...]");
            s.push_str(&message[find_char_boundary(&message, len-10)..]);
            s
        },
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
                print_lastline(&get_term_width_message(&format!("Scanning: {}", p_str), 10))
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
