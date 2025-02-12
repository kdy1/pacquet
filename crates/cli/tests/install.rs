pub mod _utils;
pub use _utils::*;

use assert_cmd::prelude::*;
use command_extra::CommandExtra;
use pacquet_testing_utils::{
    bin::{AddDefaultNpmrcInfo, CommandTempCwd},
    fs::{get_all_files, get_all_folders, is_symlink_or_junction},
};
use pipe_trait::Pipe;
use std::fs;

#[test]
fn should_install_dependencies() {
    let CommandTempCwd { pacquet, root, workspace, npmrc_info, .. } =
        CommandTempCwd::init().add_default_npmrc();
    let AddDefaultNpmrcInfo { store_dir, .. } = npmrc_info;

    eprintln!("Creating package.json...");
    let manifest_path = workspace.join("package.json");
    let package_json_content = serde_json::json!({
        "dependencies": {
            "is-odd": "3.0.1",
        },
        "devDependencies": {
            "fast-decode-uri-component": "1.0.1",
        },
    });
    fs::write(&manifest_path, package_json_content.to_string()).expect("write to package.json");

    eprintln!("Executing command...");
    pacquet.with_arg("install").assert().success();

    eprintln!("Make sure the package is installed");
    assert!(is_symlink_or_junction(&workspace.join("node_modules/is-odd")).unwrap());
    assert!(workspace.join("node_modules/.pnpm/is-odd@3.0.1").exists());

    eprintln!("Make sure it installs direct dependencies");
    assert!(!workspace.join("node_modules/is-number").exists());
    assert!(workspace.join("node_modules/.pnpm/is-number@6.0.0").exists());

    eprintln!("Make sure we install dev-dependencies as well");
    assert!(
        is_symlink_or_junction(&workspace.join("node_modules/fast-decode-uri-component")).unwrap()
    );
    assert!(workspace.join("node_modules/.pnpm/fast-decode-uri-component@1.0.1").is_dir());

    eprintln!("Snapshot");
    let workspace_folders = get_all_folders(&workspace);
    let store_files = get_all_files(&store_dir);
    insta::assert_debug_snapshot!((workspace_folders, store_files));

    drop(root); // cleanup
}

#[test]
fn should_install_exec_files() {
    let CommandTempCwd { pacquet, root, workspace, npmrc_info, .. } =
        CommandTempCwd::init().add_default_npmrc();
    let AddDefaultNpmrcInfo { store_dir, .. } = npmrc_info;

    eprintln!("Creating package.json...");
    let manifest_path = workspace.join("package.json");
    let package_json_content = serde_json::json!({
        "dependencies": {
            "pretty-exec": "0.3.10",
        },
    });
    fs::write(&manifest_path, package_json_content.to_string()).expect("write to package.json");

    eprintln!("Executing command...");
    pacquet.with_arg("install").assert().success();

    eprintln!("Listing all files in the store...");
    let store_files = get_all_files(&store_dir);

    #[cfg(unix)]
    {
        use pacquet_testing_utils::fs::is_path_executable;
        use pretty_assertions::assert_eq;
        use std::{fs::File, iter::repeat, os::unix::fs::MetadataExt};

        eprintln!("All files that end with '-exec' are executable, others not");
        let (suffix_exec, suffix_other) =
            store_files.iter().partition::<Vec<_>, _>(|path| path.ends_with("-exec"));
        let (mode_exec, mode_other) = store_files
            .iter()
            .partition::<Vec<_>, _>(|name| store_dir.join(name).as_path().pipe(is_path_executable));
        assert_eq!((&suffix_exec, &suffix_other), (&mode_exec, &mode_other));

        eprintln!("All executable files have mode 755");
        let actual_modes: Vec<_> = mode_exec
            .iter()
            .map(|name| {
                let mode = store_dir
                    .join(name)
                    .pipe(File::open)
                    .expect("open file to get mode")
                    .metadata()
                    .expect("get metadata")
                    .mode();
                (name.as_str(), mode & 0o777)
            })
            .collect();
        let expected_modes: Vec<_> =
            mode_exec.iter().map(|name| name.as_str()).zip(repeat(0o755)).collect();
        assert_eq!(&actual_modes, &expected_modes);
    }

    eprintln!("Snapshot");
    insta::assert_debug_snapshot!(store_files);

    drop(root); // cleanup
}

#[test]
fn should_install_index_files() {
    let CommandTempCwd { pacquet, root, workspace, npmrc_info, .. } =
        CommandTempCwd::init().add_default_npmrc();
    let AddDefaultNpmrcInfo { store_dir, .. } = npmrc_info;

    eprintln!("Creating package.json...");
    let manifest_path = workspace.join("package.json");
    let package_json_content = serde_json::json!({
        "dependencies": {
            "is-odd": "3.0.1",
        },
        "devDependencies": {
            "fast-decode-uri-component": "1.0.1",
        },
    });
    fs::write(&manifest_path, package_json_content.to_string()).expect("write to package.json");

    eprintln!("Executing command...");
    pacquet.with_arg("install").assert().success();

    eprintln!("Snapshot");
    let index_file_contents = index_file_contents(&store_dir);
    insta::assert_yaml_snapshot!(index_file_contents);

    drop(root); // cleanup
}
