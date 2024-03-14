use tempfile::tempdir;
use peaceful::convert;

#[test]
fn convert_single() {
    convert("./testdata/**/*.png", "./tempdir", "jpg");
}
