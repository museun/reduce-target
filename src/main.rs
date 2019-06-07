use gumdrop::Options as _;
use indexmap::IndexSet;

use std::path::{Path, PathBuf};

macro_rules! abort {
    ($f:expr, $($args:expr),* $(,)?) => {
        abort!(format!($f, $($args),*))
    };
    ($f:expr) => {{
        eprintln!("{}", $f);
        std::process::exit(1)
    }};
}

#[derive(Debug, Clone, gumdrop::Options)]
struct Options {
    #[options(help = "show this message")]
    help: bool,

    #[options(help = "root directory to search")]
    directory: Option<String>,

    #[options(help = "prints directory statistics", short = "s")]
    stats: bool,

    #[options(help = "sweeps all directories", no_short)]
    sweep: bool,
}

impl Options {
    fn get_dir(&self) -> PathBuf {
        let dir = self.directory.clone().unwrap_or_else(|| ".".into());
        let dir = Path::new(&dir).canonicalize().unwrap_or_else(|e| {
            match e.kind() {
                std::io::ErrorKind::NotFound => abort!("cannot find directory: `{}`", dir),
                _ => abort!("unknown err: {:#?}", e),
            };
        });

        match (dir.exists(), dir.is_dir()) {
            (false, _) => abort!("directory doesn't exist: `{}`", dir.display()),
            (_, false) => abort!("invalid directory: `{}`", dir.display()),
            _ => dir,
        }
    }
}

fn humanize(d: u64) -> String {
    const SIZES: [char; 9] = ['B', 'K', 'M', 'G', 'T', 'P', 'E', 'Z', 'Y'];

    let mut order = 0;
    let mut size = d as f64;
    while size >= 1024.0 && order + 1 < SIZES.len() {
        order += 1;
        size /= 1024.0;
    }

    format!("{:.2} {}", size, SIZES[order])
}

fn commaize(d: u64) -> String {
    use std::fmt::Write;
    fn comma(n: u64, s: &mut String) {
        if n < 1000 {
            write!(s, "{}", n).unwrap();
            return;
        }
        comma(n / 1000, s);
        write!(s, ",{:03}", n % 1000).unwrap();
    }

    let mut buf = String::new();
    comma(d, &mut buf);
    buf
}

fn find_targets(root: impl AsRef<Path>) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut paths = vec![];
    for dir in root.as_ref().read_dir()?.flatten().filter_map(|fi| {
        fi.file_type()
            .ok()
            .and_then(|f| if f.is_dir() { Some(fi) } else { None })
    }) {
        if dir.file_name() == "target" {
            paths.push(dir.path());
        } else {
            paths.append(&mut find_targets(dir.path())?);
        }
    }
    Ok(paths)
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Hash, Eq, PartialOrd, Ord)]
struct Record {
    size: u64,
    files: u64,
    directories: u64,
}

fn sum_targets(targets: &[PathBuf]) -> IndexSet<(&PathBuf, Record)> {
    fn sum_target(path: &PathBuf) -> Record {
        walkdir::WalkDir::new(path)
            .into_iter()
            .flatten()
            .fold(Record::default(), |mut rec, t| {
                let ty = t.file_type();
                match (ty.is_file(), ty.is_dir()) {
                    (true, _) => {
                        if let Ok(md) = t.metadata() {
                            rec.size += md.len();
                            rec.files += 1;
                        }
                    }
                    (_, true) => rec.directories += 1,
                    _ => {}
                }
                rec
            })
    }

    let mut set = IndexSet::with_capacity(targets.len());
    for target in targets {
        set.insert((target, sum_target(target)));
    }
    set
}

fn main() -> Result<(), std::io::Error> {
    let opts = Options::parse_args_default_or_exit();

    let root = opts.get_dir();
    let targets = find_targets(&root)?;
    let sums = sum_targets(&targets);

    if opts.stats || !opts.sweep {
        // clone so we can sort it, but still have it unsorted later
        let mut list = sums.clone();
        list.sort_by(|(_, l), (_, r)| l.cmp(&r).reverse());

        let mut size = 0;
        let mut files = 0;
        let mut dirs = 0;

        for (k, v) in &list {
            size += v.size;
            files += v.files;
            dirs += v.directories;

            println!("{}", k.to_str().unwrap());
            println!("{: >5}: {: >10}", "size", humanize(v.size));
            println!("{: >5}: {: >10}", "files", commaize(v.files));
            println!("{: >5}: {: >10}", "dirs", commaize(v.directories));
        }

        println!("{}", "-".repeat(30));
        println!("in {} top-level directories:", list.len());
        println!("{: >5}: {: >10}", "size", humanize(size));
        println!("{: >5}: {: >10}", "files", commaize(files));
        println!("{: >5}: {: >10}", "dirs", commaize(dirs));

        if !opts.sweep {
            return Ok(());
        }
    }

    if opts.sweep {
        for dir in sums.iter().map(|(dir, _)| (dir.to_str())).flatten() {
            match std::fs::remove_dir_all(dir) {
                Ok(..) => println!("removed: {}", dir),
                Err(err) => eprintln!("could not remove: {} because {}", dir, err),
            }
        }
    }

    Ok(())
}
