
# img2pdf

`img2pdf` is a simple rust library for converting <b>an image into a PDF file</b>. It supports handling image files with transparency and compressing them into PDF documents. It's really simple. 


## Usage

### Converting an Image from Bytes

To convert an image provided as a byte array to a PDF in memory:

```rust
use img2pdf::img2pdf_from_bytes;

let image_data = std::fs::read("sample_image.jpg").expect("Failed to read image");
let pdf_data = img2pdf_from_bytes(&image_data).expect("Failed to convert image to PDF");

std::fs::write("output.pdf", pdf_data).expect("Failed to write PDF file");
```

### Converting an Image File to a PDF File

To convert an image file directly to a PDF file:

```rust
use img2pdf::img2pdf_file;

img2pdf_file("sample_image.png", "output.pdf").expect("Failed to convert image to PDF");
```

### Command Line Interface

There is also a CLI option, but it may not be relevant for your use case

```bash
img2pdf <input_image> <output_pdf>
```