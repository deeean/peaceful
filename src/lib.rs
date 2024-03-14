use std::path::{Path, PathBuf};
use glob::glob;
use image::{DynamicImage, GenericImageView, guess_format};
use indicatif::{ParallelProgressIterator};
use rayon::iter::{ParallelIterator, IntoParallelRefIterator};

#[derive(Debug)]
enum Size {
    Numeric(u32),
    Percentage(f32),
}

fn compress_png(path: &PathBuf, output_path: &PathBuf, min_quality: u8, max_quality: u8) {
    let mut liq = imagequant::new();
    if let Err(e) = liq.set_speed(3) {
        eprintln!("❌  Failed to set speed: {}", e);
        return;
    }

    if let Err(e) = liq.set_quality(min_quality, max_quality) {
        eprintln!("❌  Failed to set quality: {}", e);
        return;
    }

    let img = match lodepng::decode32_file(&path) {
        Ok(img) => img,
        Err(e) => {
            eprintln!("❌  Failed to decode image [{}]: {}", path.display(), e);
            return;
        },
    };

    let width = img.width;
    let height = img.height;

    let mut buffer = Vec::new();

    for p in img.buffer {
        buffer.push(rgb::RGBA::new(p.r, p.g, p.b, p.a));
    }

    let mut img = match liq.new_image(buffer, img.width, img.height, 0.0) {
        Ok(img) => img,
        Err(e) => {
            eprintln!("❌  Failed to create image: {}", e);
            return;
        },
    };

    let mut res = match liq.quantize(&mut img) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("❌  Failed to quantize image: {}", e);
            return;
        },
    };

    if let Err(e) = res.set_dithering_level(1.0) {
        eprintln!("❌  Failed to set dithering level: {}", e);
        return;
    }

    match res.remapped(&mut img) {
        Ok((palette, pixels)) => {
            let mut buffer = Vec::new();

            for p in pixels {
                let c = palette[p as usize];
                buffer.push(c);
            }

            if let Err(e) = lodepng::encode32_file(output_path, &buffer, width, height) {
                eprintln!("❌  Failed to encode image: {}", e);
            }
        },
        Err(e) => eprintln!("❌  Failed to remap image: {}", e),
    }
}

pub fn compress(input: &str, output: &str, quality: u8) {
    let (min, max) = match quality {
        1 => (20, 45),
        2 => (25, 50),
        3 => (30, 70),
        4 => (40, 80),
        5 => (45, 85),
        _ => {
            eprintln!("❌  Invalid quality level 1-5");
            return;
        }
    };

    let entries = match get_image_entries(input) {
        Ok(entries) => entries,
        Err(error) => panic!("{}", error),
    };

    let output = PathBuf::from(output);
    let first_dir = extract_first_dir(input);

    entries.par_iter().progress().for_each(|entry| {
        let output_path = construct_output_path(&entry, &output, first_dir);
        if let Err(e) = create_output_dir(&output_path) {
            eprintln!("{}", e);
            return;
        }

        match imghdr::from_file(&entry) {
            Ok(Some(imghdr::Type::Png)) => compress_png(&entry, &output_path, min, max),
            _ => {
                eprintln!("❌  Unsupported image format [{}]", entry.display());
                return;
            },
        };
    });
}

pub fn resize(input: &str, output: &str, size_str: &str) {
    let entries = match get_image_entries(input) {
        Ok(entries) => entries,
        Err(error) => panic!("{}", error),
    };

    let output = PathBuf::from(output);
    let first_dir = extract_first_dir(input);

    let parsed_size = parse_size_str(size_str).expect("❌  Invalid size string");

    entries.par_iter().progress().for_each(|entry| {
        let img = match image::open(&entry) {
            Ok(img) => img,
            Err(_) => {
                eprintln!("❌  Failed to open image [{}]", entry.display());
                return;
            },
        };

        let (width, height) = img.dimensions();
        let new_width = calc_new_dim(width, &parsed_size[0]);
        let new_height = calc_new_dim(height, &parsed_size[1]);
        let resized_img = img.resize_exact(new_width, new_height, image::imageops::FilterType::Lanczos3);

        let output_path = construct_output_path(&entry, &output, first_dir);
        if let Err(e) = create_output_dir(&output_path) {
            eprintln!("{}", e);
            return;
        }

        match save_image(resized_img, output_path) {
            Ok(_) => println!("✅  Resized image [{}]", entry.display()),
            Err(e) => eprintln!("{}", e),
        };
    });
}

pub fn convert(input: &str, output: &str, format_str: &str) {
    let entries = match get_image_entries(input) {
        Ok(entries) => entries,
        Err(error) => panic!("{}", error),
    };

    let output = PathBuf::from(output);
    let first_dir = extract_first_dir(input);

    entries.par_iter().progress().for_each(|entry| {
        let img = match image::open(&entry) {
            Ok(img) => img,
            Err(_) => {
                eprintln!("❌  Failed to open image [{}]", entry.display());
                return;
            },
        };

        let output_path = construct_output_path(&entry, &output, first_dir).with_extension(format_str);
        if let Err(e) = create_output_dir(&output_path) {
            eprintln!("{}", e);
            return;
        }

        match save_image(img, output_path) {
            Ok(_) => println!("✅  Converted image [{}]", entry.display()),
            Err(e) => eprintln!("{}", e),
        }
    });
}

fn extract_first_dir(path: &str) -> &str {
    let trimmed_path = path.trim_start_matches("./").trim_start_matches('/');

    let first_directory = trimmed_path.split('/')
        .find(|&component| !component.is_empty());

    first_directory.unwrap_or_else(|| "")
}

fn create_output_dir(output_path: &Path) -> Result<(), String> {
    let output_dir = output_path.parent().ok_or(format!("❌  Failed to get parent directory of [{}]", output_path.display()))?;
    std::fs::create_dir_all(output_dir).map_err(|_| format!("❌  Failed to create directory [{}]", output_dir.display()))?;
    Ok(())
}

fn save_image(img: image::DynamicImage, output_path: PathBuf) -> Result<(), String> {
    img.save(output_path.clone()).map_err(|e| format!("❌  Failed to save image [{}]: {}", output_path.display(), e))?;
    Ok(())
}

fn get_image_entries(pattern: &str) -> Result<Vec<PathBuf>, String> {
    let entries = glob(pattern).map_err(|_| "❌  Failed to read glob pattern")?
        .filter_map(Result::ok)
        .collect::<Vec<_>>();
    if entries.is_empty() {
        Err("❌  No images found".into())
    } else {
        Ok(entries)
    }
}

fn parse_size_str(size_str: &str) -> Result<Vec<Size>, &'static str> {
    size_str.split(':')
        .map(|part| {
            if part.ends_with('%') {
                part.trim_end_matches('%').parse::<f32>().ok()
                    .map(|p| Size::Percentage(p / 100.0))
                    .ok_or("Invalid percentage")
            } else {
                part.parse::<u32>().ok()
                    .map(Size::Numeric)
                    .ok_or("Invalid number")
            }
        })
        .collect()
}

fn calc_new_dim(original_dim: u32, size: &Size) -> u32 {
    match size {
        Size::Numeric(n) => *n,
        Size::Percentage(p) => (original_dim as f32 * p) as u32,
    }
}

fn construct_output_path(entry: &PathBuf, output: &PathBuf, first_dir: &str) -> PathBuf {
    let stripped = entry.strip_prefix(first_dir).unwrap_or(entry);
    output.join(stripped)
}
