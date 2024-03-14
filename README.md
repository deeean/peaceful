# Peaceful
Peaceful is an image processing CLI tool designed for efficiency and ease of use. It supports image conversion, compression, and resizing.

## Usage
To convert PNG images to JPG format, use the following command:

```bash
peaceful convert --input "./testdata/**/*.png" --output ./output --format jpg
```

To resize images to a specific width and height, use the following command:

```bash
peaceful resize --input "./testdata/**/*.png" --output ./output --size 100:100
peaceful resize --input "./testdata/**/*.png" --output ./output --size 50%:50%
```
