use gumdrop::Options as _;
use indexmap::IndexSet;
use rayon::prelude::*;

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, gumdrop::Options)]
struct Options {
    #[options(help = "show this message")]
    help: bool,

    #[options(help = "root directory to search")]
    directory: Option<String>,

    #[options(
        help = "target kind. [docs (d, doc), release (rel), debug (dbg)]",
        short = "k"
    )]
    kind: Vec<TargetKind>,

    #[options(help = "prints directory statistics", short = "s")]
    stats: bool,

    #[options(help = "sweeps all directories", no_short)]
    sweep: bool,
}

// TODO allow to narrowing to 'deps' or 'examples' inside target/{debug,release}
#[derive(Debug, Copy, Clone)]
enum TargetKind {
    Docs,
    Release,
    Debug,
    All,
}

impl TargetKind {
    fn as_str(self) -> Option<&'static str> {
        match self {
            Self::Docs => "doc",
            Self::Release => "release",
            Self::Debug => "debug",
            Self::All => return None,
        }
        .into()
    }
}

impl std::str::FromStr for TargetKind {
    type Err = String;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let ok = match input {
            "d" | "doc" | "docs" => Self::Docs,
            "rel" | "release" => Self::Release,
            "dbg" | "debug" => Self::Debug,
            "all" => Self::All,
            e => return Err(format!("unknown target kind: '{}'", e)),
        };
        Ok(ok)
    }
}

impl Default for TargetKind {
    fn default() -> Self {
        Self::All
    }
}

impl Options {
    fn get_dir(&self) -> anyhow::Result<PathBuf> {
        let dir = self.directory.clone().unwrap_or_else(|| ".".into());
        let dir = Path::new(&dir).canonicalize().map_err(|err| {
            if let std::io::ErrorKind::NotFound = err.kind() {
                return anyhow::anyhow!("cannot find directory: `{}`", dir.escape_debug());
            };
            anyhow::Error::new(err)
        })?;

        match (dir.exists(), dir.is_dir()) {
            (false, _) => anyhow::bail!("directory doesn't exist: `{}`", dir.display()),
            (_, false) => anyhow::bail!("invalid directory: `{}`", dir.display()),
            _ => Ok(dir),
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

fn find_targets(root: impl AsRef<Path>, kind: &[TargetKind]) -> anyhow::Result<Vec<PathBuf>> {
    let subdirs = kind.iter().filter_map(|s| s.as_str()).collect::<Vec<_>>();

    let mut paths = vec![];
    for dir in root.as_ref().read_dir()?.flatten().filter_map(|fi| {
        let f = fi.file_type().ok()?;
        Some(fi).filter(|_| f.is_dir())
    }) {
        if dir.file_name() != "target" {
            paths.append(&mut find_targets(dir.path(), &kind)?);
            continue;
        }

        if subdirs.is_empty() {
            paths.push(dir.path());
            continue;
        }

        for inner in dir.path().read_dir()?.flatten() {
            if !inner.file_type()?.is_dir() {
                continue;
            }

            if let Some(dir) = inner.file_name().to_str() {
                if subdirs.contains(&dir) {
                    paths.push(inner.path());
                }
            }
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
        jwalk::WalkDir::new(path)
            .into_iter()
            .flatten()
            .fold(Record::default(), |mut rec, ty| {
                let ft = ty.file_type();
                if ft.is_file() {
                    if let Ok(md) = ty.metadata() {
                        rec.size += md.len();
                        rec.files += 1;
                    }
                }
                if ft.is_dir() {
                    rec.directories += 1
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

fn print_stats<'a>(stats: impl Iterator<Item = &'a (&'a PathBuf, Record)> + 'a, len: usize) {
    let mut size = 0;
    let mut files = 0;
    let mut dirs = 0;

    for (k, v) in stats {
        size += v.size;
        files += v.files;
        dirs += v.directories;

        println!("{}", fix_display_path(k));
        println!("{: >5}: {: >10}", "size", humanize(v.size));
        println!("{: >5}: {: >10}", "files", commaize(v.files));
        println!("{: >5}: {: >10}", "dirs", commaize(v.directories));
    }

    println!("{}", "-".repeat(30));
    println!("in {} top-level directories:", len);
    println!("{: >5}: {: >10}", "size", humanize(size));
    println!("{: >5}: {: >10}", "files", commaize(files));
    println!("{: >5}: {: >10}", "dirs", commaize(dirs));
}

fn fix_display_path(path: impl Into<PathBuf>) -> String {
    let path = path.into();
    if cfg!(target_os = "windows") {
        path.display().to_string().replace(r"\\?\", "")
    } else {
        path.display().to_string()
    }
}

fn main() -> anyhow::Result<()> {
    let opts = Options::parse_args_default_or_exit();
    let root = opts.get_dir()?;

    let kinds = opts
        .kind
        .iter()
        .filter_map(|k| k.as_str())
        .collect::<Vec<_>>();

    let kind = if !kinds.is_empty() {
        kinds.join(",")
    } else {
        "top-level target".into()
    };

    println!(
        "looking for `{}` recursively under `{}`",
        kind,
        fix_display_path(&root),
    );

    let targets = find_targets(&root, &opts.kind)?;
    let sums = sum_targets(&targets);

    if opts.stats || !opts.sweep {
        // clone so we can sort it, but still have it unsorted later
        let mut list = sums.clone();
        // TODO sort by cached key and std::cmp::Reverse
        list.sort_by(|(_, l), (_, r)| l.cmp(&r).reverse());
        print_stats(list.iter(), list.len());
    }

    if opts.sweep {
        sums.into_par_iter()
            .map(|(dir, _)| dir.to_str())
            .flatten()
            .for_each(|dir| match std::fs::remove_dir_all(dir) {
                Ok(..) => println!("removed: {}", fix_display_path(dir)),
                Err(err) => eprintln!(
                    "could not remove: {} because {}",
                    fix_display_path(dir),
                    err
                ),
            });
    }

    Ok(())
}
