use argparse::{ArgumentParser, StoreTrue, Store};

pub struct Options {
    pub version: bool,
    pub verbose: bool,
    pub dry_run: bool,
    pub search_text: String,
    pub replace_text: String,
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

    pub fn new_and_parse_args() -> Options {
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
