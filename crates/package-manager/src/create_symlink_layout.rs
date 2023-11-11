use crate::symlink_package;
use futures_util::future::join_all;
use pacquet_lockfile::{PackageSnapshotDependency, PkgName, PkgNameVerPeer};
use std::{collections::HashMap, path::Path};
use tokio::task::spawn_blocking;

/// Create symlink layout of dependencies for a package in a virtual dir.
///
/// **NOTE:** `virtual_node_modules_dir` is assumed to already exist.
pub async fn create_symlink_layout(
    dependencies: &HashMap<PkgName, PackageSnapshotDependency>,
    virtual_root: &Path,
    virtual_node_modules_dir: &Path,
) {
    let _ = join_all(dependencies.iter().map(|(name, spec)| {
        let virtual_store_name = match spec {
            PackageSnapshotDependency::PkgVerPeer(ver_peer) => {
                let package_specifier = PkgNameVerPeer::new(name.clone(), ver_peer.clone()); // TODO: remove copying here
                package_specifier.to_virtual_store_name()
            }
            PackageSnapshotDependency::DependencyPath(dependency_path) => {
                dependency_path.package_specifier.to_virtual_store_name()
            }
        };
        let name_str = name.to_string();
        let symlink_target =
            virtual_root.join(virtual_store_name).join("node_modules").join(&name_str);
        let symlink_path = virtual_node_modules_dir.join(&name_str);

        spawn_blocking(move || {
            symlink_package(&symlink_target, &symlink_path).expect("symlink pkg successful");
            // TODO: properly propagate this error
        })
    }))
    .await;
}
