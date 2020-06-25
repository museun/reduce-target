use crate::util::{commaize, fix_display_path, humanize};
use indexmap::IndexSet;
use std::path::PathBuf;

#[derive(Default, Debug, Copy, Clone, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub struct Record {
    pub size: u64,
    pub files: u64,
    pub directories: u64,
}

pub fn sum_targets(targets: &[PathBuf]) -> IndexSet<(&PathBuf, Record)> {
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

// TODO redo this
pub fn print_stats<'a>(stats: impl Iterator<Item = &'a (&'a PathBuf, Record)> + 'a, len: usize) {
    let (mut size, mut files, mut dirs) = <_>::default();

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
