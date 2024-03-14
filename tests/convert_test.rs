use peaceful::convert;

#[test]
fn test_convert() {
    convert("./testdata/**/*.png", "./tempdir/", "jpg");
}
