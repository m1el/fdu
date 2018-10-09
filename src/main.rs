extern crate walkdir;
extern crate same_file;
use walkdir::{WalkDir};
use same_file::{Handle, Key};
use ::std::collections::{HashSet};

struct Opts {
    filter_dups: bool,
    flamegraph: bool,
    list_files: bool,
    reverse: bool,
    truncate: bool,
}

impl Opts {
    fn new() -> Opts {
        Opts {
            filter_dups: false,
            flamegraph: false,
            list_files: false,
            reverse: false,
            truncate: false,
        }
    }
}

fn main() {
    let mut roots = Vec::<String>::new();
    let mut skip_arg = false;
    let mut opts = Opts::new();
    for arg in ::std::env::args().skip(1) {
        if skip_arg || !arg.starts_with('-') {
            roots.push(arg);
            continue;
        }
        for c in arg.chars().skip(1) {
            match c {
                'd' => { opts.filter_dups = true; },
                'f' => {
                    opts.flamegraph = true;
                    opts.list_files = true;
                },
                'l' => { opts.list_files = true; },
                'r' => { opts.reverse = true; }
                't' => { opts.truncate = true; }
                '-' => { skip_arg = true; },
                _ => {},
            }
        }
    }

    if roots.is_empty() {
        roots.push(".".to_string());
    }

    for root in roots {
        let mut total = 0u64;
        let mut dir_set = HashSet::<Key>::new();
        let mut file_set = HashSet::<Key>::new();
        let iter = WalkDir::new(&root)
            .into_iter()
            .filter_entry(|entry| {
                if !opts.filter_dups { return true; }
                Handle::from_path(entry.path()).ok()
                    .and_then(|h| h.as_key())
                    .map_or(true, |key| dir_set.insert(key))
            })
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                let meta = entry.metadata().ok()?;
                if !meta.is_file() { return None; }
                let len = meta.len();
                if opts.filter_dups {
                    let key = Handle::from_path(path).ok()?.as_key()?;
                    if !file_set.insert(key.clone()) { return None; }
                }
                Some((path.to_owned(), len))
            });
        for (path, len) in iter {
            total += len;
            if opts.list_files {
                let disp;
                let mut st;
                if let Some(path_str) = path.to_str() {
                    st = path_str;
                } else {
                    disp = path.display().to_string();
                    st = &disp;
                }
                st = st.trim_left_matches(&root);
                if opts.flamegraph {
                    let show = st.replace(|c|c=='/'||c=='\\', ";");
                    println!("{1} {0}", len, show);
                } else if opts.reverse {
                    println!("{1} {0}", len, st);
                } else {
                    println!("{} {}", len, st);
                }
            }
        }
        if !opts.list_files {
            println!("{} {}", root, total);
        }
    }
}
