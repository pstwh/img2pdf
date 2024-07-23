use flate2::write::ZlibEncoder;
use flate2::Compression;
use image::{DynamicImage, GenericImageView};
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

/// Converts an image from binary data to a PDF as binary data.
///
/// # Arguments
///
/// * `img_data` - A slice of bytes representing the image data.
///
/// # Returns
///
/// A `Result` containing the PDF data as a `Vec<u8>` on success, or an `io::Error` on failure.
pub fn img2pdf_from_bytes(img_data: &[u8]) -> io::Result<Vec<u8>> {
    let img = image::load_from_memory(img_data).expect("Failed to open image");
    let (width, height) = img.dimensions();

    let (rgb_img, mask_img) = separate_rgb_and_alpha(img);

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(&rgb_img)?;
    let rgb_data = encoder.finish()?;

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(&mask_img)?;
    let mask_data = encoder.finish()?;

    let mut pdf_data = Vec::new();

    writeln!(pdf_data, "%PDF-1.4")?;

    let image_object_id = 2;
    let image_object_pos = pdf_data.len();
    writeln!(
        pdf_data,
        "{} 0 obj\n<< /Type /XObject /Subtype /Image /Width {} /Height {} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /FlateDecode /Length {} /SMask {} 0 R >>",
        image_object_id,
        width,
        height,
        rgb_data.len(),
        image_object_id + 1
    )?;
    writeln!(pdf_data, "stream")?;
    pdf_data.extend(&rgb_data);
    writeln!(pdf_data, "endstream\nendobj")?;

    let mask_object_id = image_object_id + 1;
    let mask_object_pos = pdf_data.len();
    writeln!(
        pdf_data,
        "{} 0 obj\n<< /Type /XObject /Subtype /Image /Width {} /Height {} /ColorSpace /DeviceGray /BitsPerComponent 8 /Filter /FlateDecode /Length {} >>",
        mask_object_id,
        width,
        height,
        mask_data.len()
    )?;
    writeln!(pdf_data, "stream")?;
    pdf_data.extend(&mask_data);
    writeln!(pdf_data, "endstream\nendobj")?;

    let content_stream_object_id = 5;
    let content_stream_pos = pdf_data.len();
    let content = format!(
        "q\n{} 0 0 {} 0 0 cm\n/Im{} Do\nQ",
        width, height, image_object_id
    );
    writeln!(
        pdf_data,
        "{} 0 obj\n<< /Length {} >>",
        content_stream_object_id,
        content.len()
    )?;
    writeln!(pdf_data, "stream\n{}\nendstream\nendobj", content)?;

    let page_object_id = 4;
    let page_object_pos = pdf_data.len();
    writeln!(
        pdf_data,
        "{} 0 obj\n<< /Type /Page /Parent 1 0 R /MediaBox [0 0 {} {}] /Contents {} 0 R /Resources << /XObject << /Im{} {} 0 R >> >> >>",
        page_object_id, width, height, content_stream_object_id, image_object_id, image_object_id
    )?;
    writeln!(pdf_data, "endobj")?;

    let pages_object_pos = pdf_data.len();
    writeln!(
        pdf_data,
        "1 0 obj\n<< /Type /Pages /Kids [ {} 0 R ] /Count 1 >>",
        page_object_id
    )?;
    writeln!(pdf_data, "endobj")?;

    let catalog_object_pos = pdf_data.len();
    writeln!(pdf_data, "6 0 obj\n<< /Type /Catalog /Pages 1 0 R >>")?;
    writeln!(pdf_data, "endobj")?;

    let xref_start = pdf_data.len();
    writeln!(pdf_data, "xref")?;
    writeln!(pdf_data, "0 7")?;
    writeln!(pdf_data, "0000000000 65535 f ")?;
    writeln!(pdf_data, "{:010} 00000 n ", pages_object_pos)?;
    writeln!(pdf_data, "{:010} 00000 n ", image_object_pos)?;
    writeln!(pdf_data, "{:010} 00000 n ", mask_object_pos)?;
    writeln!(pdf_data, "{:010} 00000 n ", page_object_pos)?;
    writeln!(pdf_data, "{:010} 00000 n ", content_stream_pos)?;
    writeln!(pdf_data, "{:010} 00000 n ", catalog_object_pos)?;

    writeln!(pdf_data, "trailer\n<< /Size 7 /Root 6 0 R >>")?;
    writeln!(pdf_data, "startxref\n{}", xref_start)?;
    writeln!(pdf_data, "%%EOF")?;

    Ok(pdf_data)
}

/// Separates the RGB and alpha channels of an image.
///
/// # Arguments
///
/// * `img` - The `DynamicImage` to be processed.
///
/// # Returns
///
/// A tuple containing the RGB data and the alpha channel data.
fn separate_rgb_and_alpha(img: DynamicImage) -> (Vec<u8>, Vec<u8>) {
    let rgba = img.to_rgba8();
    let mut rgb = Vec::with_capacity(rgba.len() / 4 * 3);
    let mut alpha = Vec::with_capacity(rgba.len() / 4);

    for pixel in rgba.pixels() {
        rgb.push(pixel[0]);
        rgb.push(pixel[1]);
        rgb.push(pixel[2]);
        alpha.push(pixel[3]);
    }

    (rgb, alpha)
}

/// Converts an image from a file to a PDF file.
///
/// # Arguments
///
/// * `input_path` - The path to the input image file.
/// * `output_path` - The path to the output PDF file.
///
/// # Returns
///
/// An `io::Result` indicating success or failure.
pub fn img2pdf_from_file<P: AsRef<Path>>(input_path: P, output_path: P) -> io::Result<()> {
    let mut input_file = File::open(input_path)?;
    let mut img_data = Vec::new();
    input_file.read_to_end(&mut img_data)?;

    let pdf_data = img2pdf_from_bytes(&img_data)?;

    let mut output_file = File::create(output_path)?;
    output_file.write_all(&pdf_data)?;

    Ok(())
}

#[test]
fn test_img2pdf_from_bytes() {
    let mut img_file =
        File::open("examples/sample_image.jpg").expect("Failed to open sample image");
    let mut img_data = Vec::new();
    img_file
        .read_to_end(&mut img_data)
        .expect("Failed to read image data");

    let pdf_data = img2pdf_from_bytes(&img_data).expect("Failed to convert image to PDF");

    assert!(pdf_data.starts_with(b"%PDF"));
    assert!(pdf_data.ends_with(b"%%EOF\n"));
}

#[test]
fn test_img2pdf_file() {
    let input_path = "examples/sample_image.jpg";
    let output_path = "examples/sample_image.pdf";
    img2pdf_from_file(input_path, output_path).expect("Failed to convert image to PDF");

    let mut pdf_file = File::open(output_path).expect("Failed to open output PDF");
    let mut pdf_data = Vec::new();
    pdf_file
        .read_to_end(&mut pdf_data)
        .expect("Failed to read PDF data");

    assert!(pdf_data.starts_with(b"%PDF"));
    assert!(pdf_data.ends_with(b"%%EOF\n"));
}
