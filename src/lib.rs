use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};
use glob::glob;
use image::{GenericImageView, ImageFormat};
use lazy_static::lazy_static;

lazy_static! {
    static ref FORMATS: HashMap<&'static str, ImageFormat> = {
        let mut m = HashMap::new();
        m.insert("jpg", ImageFormat::Jpeg);
        m.insert("png", ImageFormat::Png);
        m.insert("gif", ImageFormat::Gif);
        m.insert("webp", ImageFormat::WebP);
        m.insert("tif", ImageFormat::Tiff);
        m.insert("tiff", ImageFormat::Tiff);
        m.insert("bmp", ImageFormat::Bmp);
        m.insert("ico", ImageFormat::Ico);
        m
    };
}

pub fn convert(input: &str, output: &str, format_str: &str) {
    let format = match FORMATS.get(format_str) {
        Some (f) => *f,
        None => panic!("Invalid format")
    };

    let entries = match glob(input) {
        Ok (paths) => paths.map(|entry| {
            match entry {
                Ok (path) => path,
                Err (e) => panic!("Failed to read path: {}", e)
            }
        }).collect::<Vec<PathBuf>>(),
        Err (e) => panic!("Failed to read glob pattern: {}", e)
    };

    if entries.len() == 0 {
        panic!("No images found");
    }

    let output = PathBuf::from(output);

    for entry in entries {
        let base_path = match entry.parent() {
            Some (p) => p,
            None => {
                eprintln!("Failed to get parent directory");
                continue;
            }
        };

        let mut img = match image::open(&entry) {
            Ok (img) => img,
            Err (e) => {
                eprintln!("Failed to open image: {}", e);
                continue;
            }
        };

        if format == ImageFormat::Ico {
            if img.width() > 256 {
                img = img.resize(256, img.height(), image::imageops::FilterType::Lanczos3);
            }

            if img.height() > 256 {
                img = img.resize(img.width(), 256, image::imageops::FilterType::Lanczos3);
            }
        }

        let mut output_path = output.join(entry.strip_prefix(base_path).unwrap());
        output_path.set_extension(format_str);

        let output_dir = match output_path.parent() {
            Some (p) => p,
            None => {
                eprintln!("Failed to get parent directory");
                continue;
            }
        };

        match std::fs::create_dir_all(output_dir) {
            Ok (_) => (),
            Err (e) => {
                eprintln!("Failed to create directory: {}", e);
                continue;
            }
        }

        match img.save(output_path.clone()) {
            Ok (_) => (),
            Err (e) => {
                eprintln!("Failed to save image: {}", e);
                continue;
            }
        }

        println!("{} -> {}", entry.display(), output_path.display());
    }
}
