use std::path::{PathBuf};
use glob::glob;
use indicatif::{ParallelProgressIterator};
use rayon::iter::{ParallelIterator, IntoParallelRefIterator};

fn extract_first_dir(path: &str) -> &str {
    let trimmed_path = path.trim_start_matches("./").trim_start_matches('/');

    let first_directory = trimmed_path.split('/')
        .find(|&component| !component.is_empty());

    first_directory.unwrap_or_else(|| "")
}

pub fn resize(input: &str, output: &str) {

}

pub fn convert(input: &str, output: &str, format_str: &str) {
    let entries = glob(input)
        .expect("Failed to read glob pattern")
        .filter_map(|e| e.ok())
        .collect::<Vec<PathBuf>>();

    if entries.len() == 0 {
        panic!("No images found");
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
