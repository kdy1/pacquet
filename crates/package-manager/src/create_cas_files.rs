use crate::{link_file, LinkFileError};
use derive_more::{Display, Error};
use futures_util::future::try_join_all;
use miette::Diagnostic;
use pacquet_npmrc::PackageImportMethod;
use std::{
    collections::HashMap,
    ffi::OsString,
    path::{Path, PathBuf},
};
use tokio::task::spawn_blocking;

/// Error type for [`create_cas_files`].
#[derive(Debug, Display, Error, Diagnostic)]
pub enum CreateCasFilesError {
    #[diagnostic(transparent)]
    LinkFile(#[error(source)] LinkFileError),
}

/// If `dir_path` doesn't exist, create and populate it with files from `cas_paths`.
///
/// If `dir_path` already exists, do nothing.
pub async fn create_cas_files(
    import_method: PackageImportMethod,
    dir_path: &Path,
    cas_paths: &HashMap<OsString, PathBuf>,
) -> Result<(), CreateCasFilesError> {
    assert_eq!(
        import_method,
        PackageImportMethod::Auto,
        "Only PackageImportMethod::Auto is currently supported, but {dir_path:?} requires {import_method:?}",
    );

    if dir_path.exists() {
        return Ok(());
    }

    let res = try_join_all(cas_paths.iter().map(|(cleaned_entry, store_path)| {
        let store_path = store_path.clone();
        let target_link = dir_path.join(cleaned_entry);
        spawn_blocking(move || {
            link_file(&store_path, &target_link).map_err(CreateCasFilesError::LinkFile)
        })
    }))
    .await;

    // Ignore join error
    if let Ok(res) = res {
        for res in res {
            res?;
        }
    }

    Ok(())
}
