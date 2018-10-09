extern crate walkdir;
extern crate same_file;
use walkdir::{WalkDir};
use same_file::{Handle, Key};
use ::std::collections::{HashSet};

fn main() {
    let mut paths = Vec::<String>::new();
    let mut skip_arg = false;
    let mut filter_dups = false;
    let mut list_files = false;
    let mut flamegraph = false;
    let mut reverse = false;
    for arg in ::std::env::args().skip(1) {
        if skip_arg || !arg.starts_with('-') {
            paths.push(arg);
            continue;
        }
        for c in arg.chars().skip(1) {
            match c {
                'l' => { list_files = true; },
                'd' => { filter_dups = true; },
                'r' => { reverse = true; }
                'f' => {
                    flamegraph = true;
                    list_files = true;
                },
                '-' => { skip_arg = true; },
                _ => {},
            }
        }
    }

    if paths.is_empty() {
        paths.push(".".to_string());
    }

    for path in paths {
        let mut total = 0u64;
        let mut dir_set = HashSet::<Key>::new();
        let mut file_set = HashSet::<Key>::new();
        let iter = WalkDir::new(path)
            .into_iter()
            .filter_entry(|entry| {
                if !filter_dups { return true; }
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
                if filter_dups {
                    let key = Handle::from_path(path).ok()?.as_key()?;
                    if !file_set.insert(key.clone()) { return None; }
                }
                Some((path.to_owned(), len))
            });
        for (path, len) in iter {
            total += len;
            if list_files {
                if flamegraph {
                    let show = path.display().to_string()
                        .replace(|c|c=='/'||c=='\\', ";");
                    println!("{1} {0}", len, show);
                } else if reverse {
                    println!("{1} {0}", len, path.display());
                } else {
                    println!("{} {}", len, path.display());
                }
            }
        }
        if !list_files {
            println!("{} {}", path, total);
        }
    }
}
