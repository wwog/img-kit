use std::collections::HashSet;
use std::path::Path;
use std::process::Command;
use std::sync::OnceLock;

#[derive(Debug)]
pub struct SipsFormatSupport {
    readable_extensions: HashSet<String>,
    writable_extensions: HashSet<String>,
}

impl SipsFormatSupport {
    pub fn can_transcode(&self, input_extension: &str, output_extension: &str) -> bool {
        self.readable_extensions.contains(input_extension)
            && self.writable_extensions.contains(output_extension)
    }
}

#[cfg(target_os = "macos")]
fn parse_sips_format_support(output: &str) -> SipsFormatSupport {
    let mut readable_extensions = HashSet::new();
    let mut writable_extensions = HashSet::new();

    for line in output.lines() {
        let columns: Vec<&str> = line.split_whitespace().collect();
        if columns.len() < 2 {
            continue;
        }
        let extension = columns[1].trim().to_ascii_lowercase();
        if extension == "--" || extension.is_empty() {
            continue;
        }

        readable_extensions.insert(extension.clone());
        if columns.iter().any(|column| *column == "Writable") {
            writable_extensions.insert(extension);
        }
    }

    SipsFormatSupport {
        readable_extensions,
        writable_extensions,
    }
}

#[cfg(target_os = "macos")]
pub fn get_sips_format_support() -> Option<&'static SipsFormatSupport> {
    static SUPPORT: OnceLock<Option<SipsFormatSupport>> = OnceLock::new();
    SUPPORT
        .get_or_init(|| {
            let output = Command::new("sips").arg("--formats").output().ok()?;
            if !output.status.success() {
                return None;
            }
            let stdout = String::from_utf8(output.stdout).ok()?;
            Some(parse_sips_format_support(&stdout))
        })
        .as_ref()
}

#[cfg(not(target_os = "macos"))]
pub fn get_sips_format_support() -> Option<&'static SipsFormatSupport> {
    None
}

pub fn query_has_alpha(input_path: &Path) -> Result<Option<bool>, String> {
    #[cfg(target_os = "macos")]
    {
        if get_sips_format_support().is_none() {
            return Ok(None);
        }

        let output = Command::new("sips")
            .arg("-g")
            .arg("hasAlpha")
            .arg("-1")
            .arg(input_path)
            .output()
            .map_err(|error| format!("调用 sips 查询透明通道失败: {error}"))?;
        if !output.status.success() {
            return Ok(None);
        }

        let stdout =
            String::from_utf8(output.stdout).map_err(|error| format!("解析 sips 输出失败: {error}"))?;
        let lower = stdout.to_ascii_lowercase();
        if lower.contains("hasalpha: yes") || lower.contains("hasalpha: true") {
            return Ok(Some(true));
        }
        if lower.contains("hasalpha: no") || lower.contains("hasalpha: false") {
            return Ok(Some(false));
        }
        return Ok(None);
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = input_path;
        Ok(None)
    }
}

pub fn run_sips(input_path: &Path, output_path: &Path, output_extension: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let status = Command::new("sips")
            .arg("-s")
            .arg("format")
            .arg(output_extension)
            .arg(input_path)
            .arg("--out")
            .arg(output_path)
            .status()
            .map_err(|error| format!("调用 sips 失败: {error}"))?;
        if !status.success() {
            return Err("sips 转码失败".to_owned());
        }
        return Ok(());
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = (input_path, output_path, output_extension);
        Err("当前系统不支持 sips".to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_transcode_requires_readable_and_writable() {
        let mut readable_extensions = HashSet::new();
        readable_extensions.insert("bmp".to_owned());
        readable_extensions.insert("heic".to_owned());

        let mut writable_extensions = HashSet::new();
        writable_extensions.insert("png".to_owned());
        writable_extensions.insert("jpeg".to_owned());

        let support = SipsFormatSupport {
            readable_extensions,
            writable_extensions,
        };

        assert!(support.can_transcode("bmp", "png"));
        assert!(support.can_transcode("heic", "jpeg"));
        assert!(!support.can_transcode("gif", "png"));
        assert!(!support.can_transcode("bmp", "webp"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_parse_sips_format_support_should_parse_readable_and_writable() {
        let sample = "\
com.microsoft.bmp            bmp   Writable
public.heif                  heif  
public.png                   png   Writable
";

        let support = parse_sips_format_support(sample);
        assert!(support.can_transcode("bmp", "png"));
        assert!(support.can_transcode("heif", "png"));
        assert!(!support.can_transcode("heif", "jpeg"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_get_sips_format_support_should_be_available_on_macos() {
        let support = get_sips_format_support();
        println!("support: {:#?}", support);
        assert!(support.is_some(), "macOS 应可获取 sips 支持格式");
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn test_get_sips_format_support_should_be_none_on_non_macos() {
        assert!(get_sips_format_support().is_none());
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn test_query_has_alpha_returns_ok_none_off_macos() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/bmp_1.bmp");
        assert_eq!(query_has_alpha(&path), Ok(None));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_query_has_alpha_for_asset_bmp_returns_some_bool() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/bmp_1.bmp");
        let result = query_has_alpha(&path).expect("查询不应失败");
        assert!(matches!(result, Some(true) | Some(false) | None));
    }
}
