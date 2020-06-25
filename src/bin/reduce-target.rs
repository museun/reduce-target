use reduce_target::{
    find_targets,
    stats::{print_stats, sum_targets},
    util::*,
    Input, Profile, TargetKind,
};

use gumdrop::Options as _;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, gumdrop::Options)]
struct Options {
    #[options(help = "show this message")]
    help: bool,

    #[options(help = "root directory to search")]
    directory: Option<String>,

    #[options(help = "profile type", short = "p")]
    profiles: Vec<Profile>,

    #[options(help = "target kind", short = "k")]
    kinds: Vec<TargetKind>,

    #[options(help = "prints directory statistics", short = "s")]
    stats: bool,

    #[options(help = "sweeps all directories", no_short)]
    sweep: bool,
}

impl Options {
    // Get the base directory (-d or implied to be `cwd`)
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

fn main() -> anyhow::Result<()> {
    let mut opts = Options::parse_args_default_or_exit();
    let root = opts.get_dir()?;

    let mut input = Input::new();

    if opts.profiles.is_empty() {
        opts.profiles.push(Profile::All)
    }

    if opts.kinds.is_empty() {
        opts.kinds.push(TargetKind::All)
    }

    for profile in &opts.profiles {
        for kind in &opts.kinds {
            input.add(*profile, *kind)
        }
    }

    println!(
        "looking recursively under `{}` for:",
        fix_display_path(&root),
    );
    println!("{}", input);

    let targets = find_targets(&root)?;
    let paths = input.filter(&targets);
    let sums = sum_targets(&paths);

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
