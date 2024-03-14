use std::path::{Path, PathBuf};
use glob::glob;
use image::GenericImageView;
use indicatif::{ParallelProgressIterator};
use rayon::iter::{ParallelIterator, IntoParallelRefIterator};

#[derive(Debug)]
enum Size {
    Numeric(u32),
    Percentage(f32),
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
            Err(_) => panic!("❌  Failed to open image [{}]", entry.display()),
        };

        let (width, height) = img.dimensions();
        let new_width = calc_new_dim(width, &parsed_size[0]);
        let new_height = calc_new_dim(height, &parsed_size[1]);
        let resized_img = img.resize_exact(new_width, new_height, image::imageops::FilterType::Lanczos3);

        let output_path = construct_output_path(&entry, &output, first_dir);
        create_output_dir(&output_path).expect("Error creating output directory");
        save_image(resized_img, output_path).expect("Error saving image");
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
            Err(_) => panic!("❌  Failed to open image [{}]", entry.display()),
        };

        let output_path = construct_output_path(&entry, &output, first_dir).with_extension(format_str);
        create_output_dir(&output_path).expect("Error creating output directory");
        save_image(img, output_path).expect("Error saving image");
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
