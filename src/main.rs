extern crate getopts;
extern crate walkdir;
extern crate same_file;
use walkdir::{WalkDir};
use same_file::{Handle, Key};
use ::std::io::{self, Write, BufWriter};
use ::std::collections::{HashSet};

use getopts::{Options, HasArg};

fn main() -> Result<(), io::Error> {
    struct Opt {
        name: (&'static str, &'static str),
        help: &'static str,
        hint: &'static str,
        hasarg: HasArg,
    }

    let opts: &[Opt] = &[
        Opt {
            name: ("b", "buffer"),
            help: "buffer output",
            hint: "",
            hasarg: HasArg::No,
        },
        Opt {
            name: ("d", "duplicates"),
            help: "include multiple instances of the same file",
            hint: "",
            hasarg: HasArg::No,
        },
        Opt {
            name: ("f", "flamegraph"),
            help: "output disk usage in flamegraph-friendly format",
            hint: "",
            hasarg: HasArg::No,
        },
        Opt {
            name: ("F", "full-name"),
            help: "do not remove root from file paths",
            hint: "",
            hasarg: HasArg::No,
        },
        Opt {
            name: ("h", "help"),
            help: "print usage",
            hint: "",
            hasarg: HasArg::No,
        },
        Opt {
            name: ("l", "list"),
            help: "list each file",
            hint: "",
            hasarg: HasArg::No,
        },
        Opt {
            name: ("r", "size-first"),
            help: "print file size first",
            hint: "",
            hasarg: HasArg::No,
        },
    ];
    let mut args = std::env::args();
    let name = args.next().unwrap();
    let mut parser = Options::new();
    for opt in opts {
        parser.opt(opt.name.0, opt.name.1,
                   opt.help, opt.hint, opt.hasarg, getopts::Occur::Optional);
    }

    let matches = match parser.parse(args) {
        Ok(m) => { m }
        Err(f) => { eprintln!("{}", f); std::process::exit(1); }
    };

    if matches.opt_present("help") {
        let brief = format!("Usage: {} [options] <path>...", name);
        print!("{}", parser.usage(&brief));
        return Ok(());
    }

    let do_buffer = matches.opt_present("buffer");
    let allow_dups = matches.opt_present("duplicates");
    let flamegraph = matches.opt_present("flamegraph");
    let full_name = matches.opt_present("full-name");
    let list_files = flamegraph || matches.opt_present("list");
    let size_first = matches.opt_present("size-first");
    let mut roots = matches.free;

    let mut stdout = io::stdout();
    let mut buf_writer;
    let output: &mut dyn Write;

    if do_buffer {
        buf_writer = BufWriter::with_capacity(1024, stdout);
        output = &mut buf_writer;
    } else {
        output = &mut stdout;
    }

    if roots.is_empty() {
        roots.push(".".to_string());
    }

    let one_root = roots.len() == 1;

    for root in roots {
        let mut total = 0u64;
        let mut dir_set = HashSet::<Key>::new();
        let mut file_set = HashSet::<Key>::new();

        let iter = WalkDir::new(&root)
            .into_iter()
            .filter_entry(|entry| {
                if allow_dups { return true; }
                Handle::from_path(entry.path()).ok()
                    .and_then(|h| h.as_key())
                    .map_or(true, |key| dir_set.insert(key))
            })
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                let meta = entry.metadata().ok()?;
                if !meta.is_file() {
                    return None;
                }
                let len = meta.len();
                if !allow_dups {
                    let key = Handle::from_path(path).ok()?.as_key()?;
                    if !file_set.insert(key.clone()) { return None; }
                }
                Some((path.to_owned(), len))
            });

        for (path, len) in iter {
            total += len;
            if list_files {
                let disp;
                let mut st;

                if let Some(path_str) = path.to_str() {
                    st = path_str;
                } else {
                    disp = path.display().to_string();
                    st = &disp;
                }

                if one_root && !full_name {
                    st = st.trim_start_matches(&root);
                }

                if flamegraph {
                    let show = st.replace(|c| c == '/' || c == '\\', ";");
                    writeln!(output, "{1} {0}", len, show.trim_start_matches(";"))?;
                } else if size_first {
                    writeln!(output, "{1} {0}", len, st)?;
                } else {
                    writeln!(output, "{0} {1}", len, st)?;
                }
            }
        }

        if !list_files {
            if size_first {
                writeln!(output, "{1} {0}", root, total)?;
            } else {
                writeln!(output, "{0} {1}", root, total)?;
            }
        }
    }

    Ok(())
}
