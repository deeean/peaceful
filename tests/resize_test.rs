use peaceful::resize;

#[test]
fn test_resize() {
    resize("./testdata/**/*.png", "./tempdir/", "50%:50%");
}
