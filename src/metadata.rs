use std::io::Read;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageEncodingFormat {
    Bmp,
    Heic,
    Dng,
    Jpeg,
    Png,
    Webp,
    Tiff,
    Unsupported,
}

/// 获取图像元数据带的编码格式
/// 通过 constants.rs 中的 TRANSCDOE_SUPPROT_FORMAT 支持的格式来定义幻数，如果找到对应的幻数，则返回枚举值，枚举包含不支持

pub fn get_image_encoding_format(image_path: &str) -> ImageEncodingFormat {
    let mut file = match std::fs::File::open(image_path) {
        Ok(file) => file,
        Err(_) => return ImageEncodingFormat::Unsupported,
    };

    let mut header = [0_u8; 32];
    let bytes_read = match file.read(&mut header) {
        Ok(size) => size,
        Err(_) => return ImageEncodingFormat::Unsupported,
    };
    let image_bytes = &header[..bytes_read];
    let image_extension = std::path::Path::new(image_path)
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase());

    if image_bytes.starts_with(b"BM") {
        return ImageEncodingFormat::Bmp;
    }

    if image_bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return ImageEncodingFormat::Jpeg;
    }

    if image_bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
        return ImageEncodingFormat::Png;
    }

    if image_bytes.len() >= 12 && &image_bytes[0..4] == b"RIFF" && &image_bytes[8..12] == b"WEBP" {
        return ImageEncodingFormat::Webp;
    }

    if image_bytes.starts_with(&[b'I', b'I', 0x2A, 0x00])
        || image_bytes.starts_with(&[b'M', b'M', 0x00, 0x2A])
        || image_bytes.starts_with(&[b'I', b'I', 0x2B, 0x00])
        || image_bytes.starts_with(&[b'M', b'M', 0x00, 0x2B])
    {
        if image_extension.as_deref() == Some("dng") {
            return ImageEncodingFormat::Dng;
        }
        return ImageEncodingFormat::Tiff;
    }

    if image_bytes.len() >= 12
        && &image_bytes[4..8] == b"ftyp"
        && matches!(
            &image_bytes[8..12],
            b"heic" | b"heix" | b"hevc" | b"hevx" | b"mif1" | b"msf1"
        )
    {
        return ImageEncodingFormat::Heic;
    }

    ImageEncodingFormat::Unsupported
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn expected_by_extension(path: &Path) -> Option<ImageEncodingFormat> {
        let extension = path.extension()?.to_str()?.to_ascii_lowercase();
        match extension.as_str() {
            "bmp" => Some(ImageEncodingFormat::Bmp),
            "heic" => Some(ImageEncodingFormat::Heic),
            "dng" => Some(ImageEncodingFormat::Dng),
            "jpg" | "jpeg" => Some(ImageEncodingFormat::Jpeg),
            "png" => Some(ImageEncodingFormat::Png),
            "webp" => Some(ImageEncodingFormat::Webp),
            "tif" | "tiff" => Some(ImageEncodingFormat::Tiff),
            _ => None,
        }
    }

    #[test]
    fn test_assets_images_by_extension() {
        let assets_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
        let entries = std::fs::read_dir(&assets_dir).expect("assets 目录应可读取");
        let mut checked_count = 0_usize;

        for entry_result in entries {
            let entry = entry_result.expect("读取 assets 条目失败");
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let Some(expected) = expected_by_extension(&path) else {
                continue;
            };

            let actual = get_image_encoding_format(&path.to_string_lossy());
            println!(
                "path: {}, actual: {:?}, expected: {:?}",
                path.display(),
                actual,
                expected
            );
            assert_eq!(actual, expected, "格式识别错误: {}", path.display());
            checked_count += 1;
        }

        assert!(checked_count > 0, "assets 目录中未找到可识别格式的测试图片");
    }

    #[test]
    fn get_image_encoding_format_missing_file_is_unsupported() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("img-kit-no-such-image-9f3a2c1d.bin");
        assert_eq!(
            get_image_encoding_format(&path.to_string_lossy()),
            ImageEncodingFormat::Unsupported
        );
    }

    #[test]
    fn get_image_encoding_format_detects_magic_bytes_in_temp_files() {
        let dir = std::env::temp_dir().join(format!(
            "img-kit-meta-magic-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).expect("temp dir");

        let png_path = dir.join("x.bin");
        std::fs::write(
            &png_path,
            [0x89_u8, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A],
        )
        .expect("write png header");
        assert_eq!(
            get_image_encoding_format(&png_path.to_string_lossy()),
            ImageEncodingFormat::Png
        );

        let jpeg_path = dir.join("x.dat");
        std::fs::write(&jpeg_path, [0xFF_u8, 0xD8, 0xFF, 0xE0]).expect("write jpeg header");
        assert_eq!(
            get_image_encoding_format(&jpeg_path.to_string_lossy()),
            ImageEncodingFormat::Jpeg
        );

        let bmp_path = dir.join("n.bmp");
        std::fs::write(&bmp_path, *b"BM....").expect("write bmp header");
        assert_eq!(
            get_image_encoding_format(&bmp_path.to_string_lossy()),
            ImageEncodingFormat::Bmp
        );

        let mut webp = [0_u8; 12];
        webp[0..4].copy_from_slice(b"RIFF");
        webp[8..12].copy_from_slice(b"WEBP");
        let webp_path = dir.join("a.webp");
        std::fs::write(&webp_path, webp).expect("write webp header");
        assert_eq!(
            get_image_encoding_format(&webp_path.to_string_lossy()),
            ImageEncodingFormat::Webp
        );

        let mut tiff = [0_u8; 8];
        tiff[0..4].copy_from_slice(&[b'I', b'I', 0x2A, 0x00]);
        let tiff_path = dir.join("sample.tiff");
        std::fs::write(&tiff_path, tiff).expect("write tiff header");
        assert_eq!(
            get_image_encoding_format(&tiff_path.to_string_lossy()),
            ImageEncodingFormat::Tiff
        );

        let dng_path = dir.join("raw.dng");
        std::fs::write(&dng_path, tiff).expect("write dng-as-tiff header");
        assert_eq!(
            get_image_encoding_format(&dng_path.to_string_lossy()),
            ImageEncodingFormat::Dng
        );

        let mut heic = [0_u8; 12];
        heic[4..8].copy_from_slice(b"ftyp");
        heic[8..12].copy_from_slice(b"heic");
        let heic_path = dir.join("c.heic");
        std::fs::write(&heic_path, heic).expect("write heic header");
        assert_eq!(
            get_image_encoding_format(&heic_path.to_string_lossy()),
            ImageEncodingFormat::Heic
        );

        let garbage_path = dir.join("garbage.bin");
        std::fs::write(&garbage_path, b"not-an-image").expect("write garbage");
        assert_eq!(
            get_image_encoding_format(&garbage_path.to_string_lossy()),
            ImageEncodingFormat::Unsupported
        );
    }
}
