#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum TargetKind {
    // Build directory
    Build,
    // Deps directory
    Deps,
    // Examples directory
    Examples,
    // Incremental cache
    Incremental,
    // Everything
    All,
}

impl TargetKind {
    pub fn as_str(self) -> Option<&'static str> {
        match self {
            Self::Build => "build",
            Self::Deps => "deps",
            Self::Examples => "examples",
            Self::Incremental => "incremental",
            Self::All => return None,
        }
        .into()
    }
}

impl Default for TargetKind {
    fn default() -> Self {
        Self::All
    }
}

impl std::str::FromStr for TargetKind {
    type Err = String;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let ok = match input {
            "b" | "build" => Self::Build,
            "d" | "deps" => Self::Deps,
            "ex" | "examples" => Self::Examples,
            "inc" | "incremental" => Self::Incremental,
            "all" => Self::All,
            e => return Err(format!("unknown target kind: '{}'", e)),
        };
        Ok(ok)
    }
}
