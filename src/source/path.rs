use std::fmt::{Display, Formatter, Result as FormatResult};
use std::fs;
use std::path::{Path, PathBuf};

use std::io::Result as IOResult;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct CanonicalPath {
    path: PathBuf,
}

impl CanonicalPath {
    pub fn from<P: AsRef<Path>>(path: &P) -> Option<Self> {
        Some(Self {
            path: fs::canonicalize(path).ok()?,
        })
    }

    pub fn join(&self, relative: RelativePath) -> Option<Self> {
        Self::from(&self.path.join(relative))
    }

    pub fn parent(&self) -> Self {
        let mut path = self.path.clone();
        path.pop();
        Self { path }
    }

    pub fn read(&self) -> IOResult<String> {
        fs::read_to_string(self)
    }
}

impl AsRef<Path> for CanonicalPath {
    fn as_ref(&self) -> &Path {
        self.path.as_path()
    }
}

impl Display for CanonicalPath {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        write!(formatter, "{}", self.path.to_string_lossy().to_string())
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct RelativePath {
    path: PathBuf,
}

impl RelativePath {
    pub fn from<P: AsRef<Path>>(path: &P) -> Option<Self> {
        let path = PathBuf::from(path.as_ref());
        if path.is_relative() {
            Some(Self { path })
        } else {
            None
        }
    }

    pub fn join(&self, relative: RelativePath) -> Option<Self> {
        Self::from(&self.path.join(relative))
    }
}

impl AsRef<Path> for RelativePath {
    fn as_ref(&self) -> &Path {
        self.path.as_path()
    }
}

impl Display for RelativePath {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        write!(formatter, "{}", self.path.to_string_lossy().to_string())
    }
}
