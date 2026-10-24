//! Shared test helpers, for tests that need to know where the crate is.

use std::path::PathBuf;

/// The crate's own directory. errand lives in a directory of errand-box's
/// repository, so the version control root is not the crate's root.
pub(crate) fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
