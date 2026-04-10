use std::path::{Path, PathBuf};
use std::process::Command;

fn img_kit_executable() -> PathBuf {
    if let Some(path) = std::env::var_os("CARGO_BIN_EXE_img_kit") {
        return PathBuf::from(path);
    }

    let mut target_dir = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("target"));
    if target_dir.ends_with("deps") {
        target_dir.pop();
    }
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".into());
    let exe_name = if cfg!(windows) { "img-kit.exe" } else { "img-kit" };
    target_dir.join(profile).join(exe_name)
}

#[test]
fn help_exits_success() {
    let status = Command::new(img_kit_executable())
        .arg("--help")
        .status()
        .expect("应能启动 img-kit");
    assert!(status.success(), "--help 应以成功退出");
}

#[test]
fn transcode_missing_args_fails() {
    let output = Command::new(img_kit_executable())
        .arg("transcode")
        .output()
        .expect("应能启动 img-kit");
    assert!(!output.status.success(), "缺少参数时应失败");
}

#[test]
fn unknown_subcommand_fails() {
    let output = Command::new(img_kit_executable())
        .arg("nope")
        .output()
        .expect("应能启动 img-kit");
    assert!(!output.status.success());
}
