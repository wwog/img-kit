/// 将图片归一化为可发送格式（jpg/png/webp）。
/// 输入已是目标格式时直接返回原路径；其他支持格式根据是否含透明通道转为 png 或 jpg。
use crate::metadata::{ImageEncodingFormat, get_image_encoding_format};
use crate::sips;
use image::{ColorType, DynamicImage, ImageFormat};
use std::path::Path;

fn has_alpha_channel(image: &DynamicImage) -> bool {
    matches!(
        image.color(),
        ColorType::La8
            | ColorType::La16
            | ColorType::Rgba8
            | ColorType::Rgba16
            | ColorType::Rgba32F
    )
}

fn choose_output_extension_by_alpha(input_path: &Path) -> Result<&'static str, String> {
    if let Some(has_alpha) = sips::query_has_alpha(input_path)? {
        if has_alpha {
            return Ok("png");
        }
        return Ok("jpg");
    }

    let image = image::open(input_path).map_err(|error| format!("读取图片失败: {error}"))?;
    if has_alpha_channel(&image) {
        Ok("png")
    } else {
        Ok("jpg")
    }
}

pub fn transcode_image(input_path: &str, output_dir: &str) -> Result<String, String> {
    let input_format = get_image_encoding_format(input_path);
    match input_format {
        ImageEncodingFormat::Jpeg | ImageEncodingFormat::Png | ImageEncodingFormat::Webp => {
            // Already a sendable format: copy into output_dir so the returned path
            // is always under output_dir, keeping the contract consistent for callers.
            std::fs::create_dir_all(output_dir)
                .map_err(|e| format!("创建输出目录失败: {e}"))?;
            let input_path_obj = Path::new(input_path);
            let file_name = input_path_obj
                .file_name()
                .ok_or_else(|| "输入路径无文件名".to_owned())?;
            let output_path = Path::new(output_dir).join(file_name);
            std::fs::copy(input_path, &output_path)
                .map_err(|e| format!("复制文件失败: {e}"))?;
            return Ok(output_path.to_string_lossy().into_owned());
        }
        ImageEncodingFormat::Unsupported => {
            return Err("输入文件格式不支持转码".to_owned());
        }
        ImageEncodingFormat::Heic
        | ImageEncodingFormat::Dng
        | ImageEncodingFormat::Bmp
        | ImageEncodingFormat::Tiff => {}
    }

    std::fs::create_dir_all(output_dir).map_err(|error| format!("创建输出目录失败: {error}"))?;
    let input_path = Path::new(input_path);
    let output_extension = choose_output_extension_by_alpha(input_path)?;
    let file_stem = input_path
        .file_stem()
        .and_then(|file_stem| file_stem.to_str())
        .filter(|file_stem| !file_stem.is_empty())
        .unwrap_or("transcoded");
    let output_path = Path::new(output_dir).join(format!("{file_stem}.{output_extension}"));

    let input_extension = input_path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase())
        .unwrap_or_default();

    let use_sips = sips::get_sips_format_support()
        .is_some_and(|support| support.can_transcode(&input_extension, output_extension));

    if use_sips {
        sips::run_sips(input_path, &output_path, output_extension)?;
    } else {
        let image = image::open(input_path).map_err(|error| format!("读取图片失败: {error}"))?;
        let output_format = if output_extension == "png" {
            ImageFormat::Png
        } else {
            ImageFormat::Jpeg
        };
        image
            .save_with_format(&output_path, output_format)
            .map_err(|error| format!("写入图片失败: {error}"))?;
    }

    Ok(output_path.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_output_dir() -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("系统时间异常")
            .as_nanos();
        std::env::temp_dir()
            .join(format!("img-kit-transcode-{timestamp}"))
            .to_string_lossy()
            .into_owned()
    }

    #[test]
    fn test_transcode_bmp_image() {
        let input_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("bmp_1.bmp")
            .to_string_lossy()
            .into_owned();
        let output_dir = unique_output_dir();

        let output_path = transcode_image(&input_path, &output_dir).expect("bmp 应可转码");
        println!("output_path: {}", output_path);
        assert!(Path::new(&output_path).exists(), "输出文件应存在");

        let output_format = get_image_encoding_format(&output_path);
        assert!(
            matches!(
                output_format,
                ImageEncodingFormat::Jpeg | ImageEncodingFormat::Png
            ),
            "输出格式应为 jpg 或 png，实际为: {:?}",
            output_format
        );
    }

    #[test]
    fn test_transcode_png_returns_output_dir_path() {
        let input_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("png_1.png")
            .to_string_lossy()
            .into_owned();
        let output_dir = unique_output_dir();

        let result = transcode_image(&input_path, &output_dir);
        assert!(result.is_ok(), "png 输入应直接复制并返回路径: {:?}", result.err());
        let returned = result.unwrap();
        assert!(
            returned.starts_with(&output_dir),
            "返回路径应在 output_dir 内，实际: {returned}"
        );
        assert!(Path::new(&returned).exists(), "输出文件应存在");
    }

    #[test]
    fn test_transcode_jpeg_returns_output_dir_path() {
        let input_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("jpg_1.jpg")
            .to_string_lossy()
            .into_owned();
        let output_dir = unique_output_dir();

        let result = transcode_image(&input_path, &output_dir);
        assert!(result.is_ok(), "jpeg 输入应直接复制并返回路径: {:?}", result.err());
        let returned = result.unwrap();
        assert!(
            returned.starts_with(&output_dir),
            "返回路径应在 output_dir 内，实际: {returned}"
        );
        assert!(Path::new(&returned).exists(), "输出文件应存在");
    }

    #[test]
    fn test_transcode_unsupported_magic_should_fail() {
        let dir = std::env::temp_dir().join(format!(
            "img-kit-transcode-unsupported-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let input_path = dir.join("unknown.bin");
        std::fs::write(&input_path, b"no-valid-magic-here").expect("write input");

        let output_dir = unique_output_dir();
        let result = transcode_image(
            &input_path.to_string_lossy(),
            &output_dir,
        );
        assert!(result.is_err(), "无法识别的格式应报错");
        assert!(
            result.unwrap_err().contains("不支持"),
            "应提示格式不支持"
        );
    }

    #[test]
    fn test_transcode_generated_bmp_roundtrip_metadata() {
        let dir = std::env::temp_dir().join(format!(
            "img-kit-transcode-gen-bmp-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let input_path = dir.join("one.bmp");
        let rgb: image::RgbImage = image::ImageBuffer::from_pixel(2, 2, image::Rgb([10_u8, 20, 30]));
        rgb.save_with_format(&input_path, ImageFormat::Bmp)
            .expect("write bmp");

        let output_dir = unique_output_dir();
        let output_path = transcode_image(
            &input_path.to_string_lossy(),
            &output_dir,
        )
        .expect("程序生成的 bmp 应可转码");

        assert!(Path::new(&output_path).exists());
        let fmt = get_image_encoding_format(&output_path);
        assert!(
            matches!(fmt, ImageEncodingFormat::Jpeg | ImageEncodingFormat::Png),
            "输出应为 jpg 或 png，实际: {fmt:?}"
        );
    }
}
