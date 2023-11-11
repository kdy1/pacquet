use crate::symlink_package;
use futures_util::future::join_all;
use pacquet_lockfile::{PkgName, PkgNameVerPeer, RootProjectSnapshot};
use pacquet_npmrc::Npmrc;
use pacquet_package_manifest::DependencyGroup;
use tokio::task::spawn_blocking;

/// This subroutine creates symbolic links in the `node_modules` directory for
/// the direct dependencies. The targets of the link are the virtual directories.
///
/// If package `foo@x.y.z` is declared as a dependency in `package.json`,
/// symlink `foo -> .pacquet/foo@x.y.z/node_modules/foo` shall be created
/// in the `node_modules` directory.
#[must_use]
pub struct SymlinkDirectDependencies<'a, DependencyGroupList>
where
    DependencyGroupList: IntoIterator<Item = DependencyGroup>,
{
    pub config: &'static Npmrc,
    pub project_snapshot: &'a RootProjectSnapshot,
    pub dependency_groups: DependencyGroupList,
}

impl<'a, DependencyGroupList> SymlinkDirectDependencies<'a, DependencyGroupList>
where
    DependencyGroupList: IntoIterator<Item = DependencyGroup>,
{
    /// Execute the subroutine.
    pub async fn run(self) {
        let SymlinkDirectDependencies { config, project_snapshot, dependency_groups } = self;

        let RootProjectSnapshot::Single(project_snapshot) = project_snapshot else {
            panic!("Monorepo is not yet supported"); // TODO: properly propagate this error
        };

        join_all(
            project_snapshot
                .dependencies_by_groups(dependency_groups)
                .collect::<Vec<_>>()
                .iter()
                .map(|(name, spec)| {
                    // TODO: the code below is not optimal
                    let virtual_store_name =
                        PkgNameVerPeer::new(PkgName::clone(name), spec.version.clone())
                            .to_virtual_store_name();

                    let name_str = name.to_string();
                    let symlink_target = config
                        .virtual_store_dir
                        .join(virtual_store_name)
                        .join("node_modules")
                        .join(&name_str);

                    let symlink_path = config.modules_dir.join(&name_str);

                    spawn_blocking(move || {
                        symlink_package(&symlink_target, &symlink_path).expect("symlink pkg");
                        // TODO: properly propagate this error
                    })
                }),
        )
        .await;
    }
}
