// NOTE: named profiles aren't stable yet: https://github.com/rust-lang/cargo/issues/6988
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Profile {
    Debug,
    Release,
    Doc,
    All,
}

impl Profile {
    pub fn as_str(self) -> Option<&'static str> {
        match self {
            Self::Debug => "debug",
            Self::Release => "release",
            Self::Doc => "doc",
            Self::All => return None,
        }
        .into()
    }
}

impl Default for Profile {
    fn default() -> Self {
        Self::All
    }
}

impl std::str::FromStr for Profile {
    type Err = String;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let ok = match input {
            "d" | "doc" | "docs" => Self::Doc,
            "rel" | "release" => Self::Release,
            "dbg" | "debug" => Self::Debug,
            "all" => Self::All,
            e => return Err(format!("unknown profile: '{}'", e)),
        };
        Ok(ok)
    }
}
