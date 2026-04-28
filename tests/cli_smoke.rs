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

#[test]
fn transcode_jpg_returns_output_dir_path_on_stdout() {
    let input_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("jpg_1.jpg");
    let output_dir = std::env::temp_dir().join(format!("img-kit-cli-smoke-{}", std::process::id()));

    let output = Command::new(img_kit_executable())
        .args(["transcode", &input_path.to_string_lossy(), &output_dir.to_string_lossy()])
        .output()
        .expect("应能启动 img-kit");

    assert!(output.status.success(), "jpg 输入应成功退出");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let result_path = Path::new(stdout.trim());
    assert!(result_path.exists(), "stdout 中的路径应指向实际文件");
    assert!(
        result_path.starts_with(&output_dir),
        "返回路径应在 output_dir 内，实际: {}",
        result_path.display()
    );
}

#[test]
fn transcode_png_returns_output_dir_path_on_stdout() {
    let input_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("png_1.png");
    let output_dir = std::env::temp_dir().join(format!("img-kit-cli-smoke-png-{}", std::process::id()));

    let output = Command::new(img_kit_executable())
        .args(["transcode", &input_path.to_string_lossy(), &output_dir.to_string_lossy()])
        .output()
        .expect("应能启动 img-kit");

    assert!(output.status.success(), "png 输入应成功退出");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let result_path = Path::new(stdout.trim());
    assert!(result_path.exists(), "stdout 中的路径应指向实际文件");
    assert!(
        result_path.starts_with(&output_dir),
        "返回路径应在 output_dir 内，实际: {}",
        result_path.display()
    );
}

#[test]
fn transcode_bmp_outputs_converted_file_path_on_stdout() {
    let input_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("bmp_1.bmp");
    let output_dir = std::env::temp_dir().join(format!("img-kit-cli-smoke-bmp-{}", std::process::id()));

    let output = Command::new(img_kit_executable())
        .args(["transcode", &input_path.to_string_lossy(), &output_dir.to_string_lossy()])
        .output()
        .expect("应能启动 img-kit");

    assert!(output.status.success(), "bmp 输入应转码成功");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let result_path = Path::new(stdout.trim());
    assert!(result_path.exists(), "stdout 中的路径应指向实际文件");
    let ext = result_path.extension().and_then(|e| e.to_str()).unwrap_or("");
    assert!(
        ext == "jpg" || ext == "png",
        "转码输出应为 jpg 或 png，实际: {ext}"
    );
}

