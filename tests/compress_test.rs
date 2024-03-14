use peaceful::compress;

#[test]
fn test_compress() {
    compress("./testdata/**/*.png", "./tempdir/", 1);
}
