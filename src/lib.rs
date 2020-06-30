use std::path::{Path, PathBuf};

mod target_kind;
pub use target_kind::TargetKind;

mod profile;
pub use profile::Profile;

mod input;
pub use input::Input;

pub mod util;

pub mod stats;

pub fn find_targets(root: &Path) -> anyhow::Result<Vec<PathBuf>> {
    if root.is_dir() && root.ends_with("target") {
        return Ok(vec![root.to_path_buf()]);
    }

    let mut out = vec![];
    fn scan(root: &Path, out: &mut Vec<PathBuf>) -> anyhow::Result<()> {
        for dir in root
            .read_dir()?
            .flatten()
            .filter_map(|fi| fi.file_type().ok().filter(|f| f.is_dir()).map(|_| fi))
        {
            let path = dir.path();
            if path.ends_with("target") {
                out.push(path);
                continue;
            }

            scan(&path, out)?;
        }
        Ok(())
    }

    scan(root, &mut out).map(|_| out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn input_resolve() {
        let root = PathBuf::from("c:/dev/rs/twitchchat/target");
        let targets = find_targets(&root).unwrap();
        eprintln!("{:#?}", targets);

        let mut input = Input::default();
        input.add(Profile::Debug, TargetKind::Incremental);
        input.add(Profile::Debug, TargetKind::Examples);
        input.add(Profile::Doc, TargetKind::All);

        for p in input.filter(&targets) {
            assert!(p.is_dir());
            eprintln!("{:?}", p);
        }

        // let root = PathBuf::from("c:/dev/rs/twitchchat/target");
        // eprintln!("{:#?}", input.resolve_directory(&root));
    }
}
