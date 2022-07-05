use std::{fs::File, io::BufWriter, path::PathBuf};

use clap::Parser;
use image::io::Reader as ImageReader;
use printpdf::{ImageTransform, ImageXObject, Mm, PdfDocument, Px};

#[derive(Parser, Debug)]
#[clap(
    author = "Jason5Lee",
    version = "0.1.0",
    about = "Convert long image to A4-page PDF"
)]
struct Cli {
    #[clap(
        required = true,
        parse(from_os_str),
        short,
        long,
        help = "the path of the input image"
    )]
    input: PathBuf,
    #[clap(
        required = true,
        parse(from_os_str),
        short,
        long,
        help = "the path of the output PDF"
    )]
    output: PathBuf,
    #[clap(required = true, short, long, help = "the title of the PDF document")]
    doc_title: String,
}

const A4_WIDTH: Mm = Mm(210.0);
const A4_HEIGHT: Mm = Mm(297.0);
const A4_WIDTH_INCH: f64 = 8.267718;

fn main() -> anyhow::Result<()> {
    let Cli {
        input: input_path,
        output: output_path,
        doc_title,
    } = Cli::parse();
    // Format other than rgb8 seems not to work.
    let img_buf = ImageReader::open(input_path)?.decode()?.into_rgb8();
    let (width, height) = img_buf.dimensions();
    let page_image_height = ((width as f32) * 2.0f32.sqrt()).floor() as u32;
    let dpi = (width as f64) / A4_WIDTH_INCH;
    let mut current = 0;

    let (pdf_doc, pdf_page1, pdf_layer1) =
        PdfDocument::new(doc_title, A4_WIDTH, A4_HEIGHT, "Layer 1");
    let mut current_layer = Some(pdf_doc.get_page(pdf_page1).get_layer(pdf_layer1));
    let mut current_page: u32 = 1;

    while current < height {
        let layer = current_layer.take().unwrap_or_else(|| {
            current_page += 1;
            let (new_page, new_layer) =
                pdf_doc.add_page(A4_WIDTH, A4_HEIGHT, format!("Layer {current_page}"));
            pdf_doc.get_page(new_page).get_layer(new_layer)
        });

        let pdf_img_buf =
            image::imageops::crop_imm(&img_buf, 0, current, width, page_image_height).to_image();
        let translate_y =
            A4_HEIGHT - A4_HEIGHT / (page_image_height as f64) * (pdf_img_buf.height() as f64);
        let pdf_img = ImageXObject {
            width: Px(pdf_img_buf.width() as usize),
            height: Px(pdf_img_buf.height() as usize),
            color_space: printpdf::ColorSpace::Rgb,
            bits_per_component: printpdf::ColorBits::Bit8,
            interpolate: true,
            image_data: pdf_img_buf.into_raw(),
            image_filter: None,
            clipping_bbox: None,
        };

        printpdf::Image::from(pdf_img).add_to_layer(
            layer,
            ImageTransform {
                dpi: Some(dpi),
                translate_y: Some(translate_y),
                ..Default::default()
            },
        );

        current += page_image_height;
    }
    pdf_doc.save(&mut BufWriter::new(File::create(output_path)?))?;
    Ok(())
}
