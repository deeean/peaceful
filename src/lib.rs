use std::path::{PathBuf};
use glob::glob;
use image::GenericImageView;
use indicatif::{ParallelProgressIterator};
use rayon::iter::{ParallelIterator, IntoParallelRefIterator};

fn extract_first_dir(path: &str) -> &str {
    let trimmed_path = path.trim_start_matches("./").trim_start_matches('/');

    let first_directory = trimmed_path.split('/')
        .find(|&component| !component.is_empty());

    first_directory.unwrap_or_else(|| "")
}

#[derive(Debug)]
enum Size {
    Numeric(u32),
    Percentage(f32),
}

pub fn resize(input: &str, output: &str, size_str: &str) {
    let entries = glob(input)
        .expect("❌  Failed to read glob pattern")
        .filter_map(|e| e.ok())
        .collect::<Vec<PathBuf>>();

    if entries.len() == 0 {
        panic!("❌  No images found");
    }

    let output = PathBuf::from(output);
    let first_dir = extract_first_dir(input);

    let parts_of_size = size_str.split(":").collect::<Vec<&str>>();
    if parts_of_size.len() < 2 {
        panic!("❌  Invalid size string");
    }

    let parsed_size = parts_of_size.iter()
        .map(|part| {
            if part.ends_with('%') {
                let percentage = part.trim_end_matches('%').parse::<f32>();
                match percentage {
                    Ok (p) => Size::Percentage(p / 100.0),
                    Err (_) => panic!("❌  Invalid percentage")
                }
            } else {
                let numeric = part.parse::<u32>();
                match numeric {
                    Ok (n) => Size::Numeric(n),
                    Err (_) => panic!("❌  Invalid number")
                }
            }
        })
        .collect::<Vec<Size>>();

    let results = entries
        .par_iter()
        .progress()
        .map(|entry| {
            let mut img = match image::open(&entry) {
                Ok (img) => img,
                Err (_) => {
                    return format!("❌  Failed to open image [{}]", entry.display());
                }
            };

            let (width, height) = img.dimensions();

            let new_width = match &parsed_size[0] {
                Size::Numeric (n) => *n,
                Size::Percentage (p) => (width as f32 * p) as u32
            };

            let new_height = match &parsed_size[1] {
                Size::Numeric (n) => *n,
                Size::Percentage (p) => (height as f32 * p) as u32
            };

            img = img.resize_exact(new_width, new_height, image::imageops::FilterType::Lanczos3);

            let stripped = match entry.strip_prefix(first_dir) {
                Ok (p) => p,
                Err (_) => entry
            };

            let mut output_path = output.join(stripped);

            let output_dir = match output_path.parent() {
                Some (p) => p,
                None => {
                    return format!("❌  Failed to get parent directory of [{}]", output_path.display());
                }
            };

            match std::fs::create_dir_all(output_dir) {
                Ok (_) => (),
                Err (_) => {
                    return format!("❌  Failed to create directory [{}]", output_dir.display());
                }
            }

            match img.save(output_path.clone()) {
                Ok (_) => (),
                Err (e) => {
                    return format!("❌  Failed to save image [{}]: {}", output_path.display(), e);
                }
            }

            format!("✅  Successfully resize [{}] to [{}]", entry.display(), output_path.display())
        })
        .collect::<Vec<String>>();

    for result in results {
        println!("{}", result);
    }
}

pub fn convert(input: &str, output: &str, format_str: &str) {
    let entries = glob(input)
        .expect("❌  Failed to read glob pattern")
        .filter_map(|e| e.ok())
        .collect::<Vec<PathBuf>>();

    if entries.len() == 0 {
        panic!("❌  No images found");
    }

    let output = PathBuf::from(output);
    let first_dir = extract_first_dir(input);

    let results = entries
        .par_iter()
        .progress()
        .map(|entry| {
            let img = match image::open(&entry) {
                Ok (img) => img,
                Err (_) => {
                    return format!("❌  Failed to open image [{}]", entry.display());
                }
            };

            let stripped = match entry.strip_prefix(first_dir) {
                Ok (p) => p,
                Err (_) => entry
            };

            let mut output_path = output.join(stripped);
            output_path.set_extension(format_str);

            let output_dir = match output_path.parent() {
                Some (p) => p,
                None => {
                    return format!("❌  Failed to get parent directory of [{}]", output_path.display());
                }
            };

            match std::fs::create_dir_all(output_dir) {
                Ok (_) => (),
                Err (_) => {
                    return format!("❌  Failed to create directory [{}]", output_dir.display());
                }
            }

            match img.save(output_path.clone()) {
                Ok (_) => (),
                Err (e) => {
                    return format!("❌  Failed to save image [{}]: {}", output_path.display(), e);
                }
            }

            format!("✅  Successfully converted [{}] to [{}]", entry.display(), output_path.display())
        })
        .collect::<Vec<String>>();

    for result in results {
        println!("{}", result);
    }
}
