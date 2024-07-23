use std::env;
use img2pdf::img2pdf_from_file;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 3 {
        eprintln!("Usage: {} <input_image> <output_pdf>", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];

    match img2pdf_from_file(input_path, output_path) {
        Ok(_) => println!("PDF created successfully: {}", output_path),
        Err(e) => eprintln!("Error creating PDF: {}", e),
    }
}
