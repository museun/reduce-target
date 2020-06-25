use crate::{Profile, TargetKind};

use std::{
    collections::{BTreeSet, HashMap},
    path::{Path, PathBuf},
};

#[derive(Default, Debug)]
pub struct Input {
    map: HashMap<Profile, Vec<TargetKind>>,
}

// TODO this isn't really that efficient
// TODO doc shouldn't have any `target kinds`
impl std::fmt::Display for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const KINDS: [&str; 4] = ["build", "deps", "examples", "incremental"];
        const PROFILES: [&str; 3] = ["debug", "release", "doc"];

        fn print<'a>(
            f: &mut std::fmt::Formatter<'_>,
            keys: impl Iterator<Item = &'a str>,
            values: impl Iterator<Item = &'a str> + Clone,
        ) -> std::fmt::Result {
            for key in keys {
                writeln!(f, "{}", key)?;
                for value in values.clone() {
                    writeln!(f, "- {}", value)?;
                }
            }
            Ok(())
        }

        if self.map.is_empty() {
            return print(f, KINDS.iter().copied(), PROFILES.iter().copied());
        }

        let (keys, values) = self.map.iter().fold(
            (BTreeSet::new(), BTreeSet::new()),
            |(mut k, mut v), (l, r)| {
                match l.as_str() {
                    Some(s) => {
                        k.insert(s);
                    }
                    None => k.extend(&PROFILES),
                };

                r.iter().for_each(|s| match s.as_str() {
                    Some(s) => {
                        v.insert(s);
                    }
                    None => v.extend(&KINDS),
                });

                (k, v)
            },
        );

        print(f, keys.iter().copied(), values.iter().copied())
    }
}

impl Input {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, profile: Profile, kind: TargetKind) {
        self.map.entry(profile).or_default().push(kind);
    }

    pub fn filter(&self, targets: &[PathBuf]) -> Vec<PathBuf> {
        targets
            .iter()
            .flat_map(|target| self.resolve_directory(target))
            .collect()
    }

    fn resolve_directory(&self, root: &Path) -> Vec<PathBuf> {
        let cap = self.map.len() + self.map.values().map(Vec::len).sum::<usize>();
        let mut out = Vec::with_capacity(cap);

        for (profile, kind) in &self.map {
            let path = profile
                .as_str()
                .map(|parent| root.join(parent))
                .unwrap_or_else(|| root.to_path_buf());

            for child in kind {
                let p = child
                    .as_str()
                    .map(|child| path.join(child))
                    .unwrap_or_else(|| path.to_path_buf());
                if !p.is_dir() {
                    // TODO warn on this
                    continue;
                }
                out.push(p)
            }
        }

        out.shrink_to_fit();
        out
    }
}
