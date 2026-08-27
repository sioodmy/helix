use std::path::PathBuf;

pub mod mini;
pub use mini::MiniExplorer as Explorer;

pub fn file_explorer(root: PathBuf, _editor: &helix_view::Editor) -> Result<Explorer, std::io::Error> {
    Ok(Explorer::new(root))
}
