use std::collections::HashMap;
use lazy_static::lazy_static;

lazy_static! {
    static ref FORMATS: HashMap<&'static str, image::ImageFormat> = {
        let mut m = HashMap::new();
        m.insert("jpg", image::ImageFormat::Jpeg);
        m.insert("jpeg", image::ImageFormat::Jpeg);
        m.insert("png", image::ImageFormat::Png);
        m.insert("gif", image::ImageFormat::Gif);
        m.insert("webp", image::ImageFormat::WebP);
        m.insert("tif", image::ImageFormat::Tiff);
        m.insert("tiff", image::ImageFormat::Tiff);
        m.insert("tga", image::ImageFormat::Tga);
        m.insert("bmp", image::ImageFormat::Bmp);
        m.insert("ico", image::ImageFormat::Ico);
        m.insert("hdr", image::ImageFormat::Hdr);
        m.insert("pnm", image::ImageFormat::Pnm);
        m.insert("dds", image::ImageFormat::Dds);
        m.insert("ff", image::ImageFormat::Farbfeld);
        m.insert("exr", image::ImageFormat::OpenExr);
        m
    };
}

fn main() {
    let matches = clap::Command::new("peaceful")
        .about("A simple image processing tool")
        .version("0.1.0")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            clap::Command::new("convert")
                .short_flag('c')
                .about("Convert images from one format to another")
                .arg(
                    clap::Arg::new("input")
                        .short('i')
                        .required(true)
                        .index(1),
                )
                .arg(
                    clap::Arg::new("output")
                        .short('o')
                        .required(true)
                        .index(2),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("convert", args)) => {
            let input = args.get_one::<String>("input").unwrap();
            let output = args.get_one::<String>("output").unwrap();

            let input_format = input.split('.').last().unwrap();
            let output_format = output.split('.').last().unwrap();

            let output_format = FORMATS.get(output_format).unwrap();

            let img = image::io::Reader::open(input).unwrap().decode().unwrap();
            img.save_with_format(output, *output_format).unwrap();
        }
        _ => unreachable!(),
    }
    // let img = image::io::Reader::open("./testdata/lenna.png").unwrap().decode().unwrap();
    // img.save_with_format("../testdata/lenna.jpg", image::ImageFormat::Jpeg).unwrap();
}
