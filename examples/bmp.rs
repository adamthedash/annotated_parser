use annotated_parser::parsers::byte::ByteParser;
use annotated_parser::prelude::*;

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct BMPFileHeader {
    signature: [u8; 2],
    file_size: u32,
    reserved1: u16,
    reserved2: u16,
    pixel_offset: u32,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct BMPInfoHeader {
    header_size: u32,
    width: u32,
    height: u32,
    color_planes: u16,
    bits_per_pixel: u16,
    compression: u32,
    image_size: u32,
    h_resolution: u32,
    v_resolution: u32,
    palette_colors: u32,
    important_colors: u32,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Bmp {
    file_header: BMPFileHeader,
    info_header: BMPInfoHeader,
    pixel_data: Vec<Vec<[u8; 4]>>,
}

fn bmp_parser() -> impl for<'a> Parser<&'a [u8], Output = Bmp> {
    let file_header = (
        b"BM".trace("signature"),
        u32::LE.trace("file_size"),
        u16::LE.trace("reserved1"),
        u16::LE.trace("reserved2"),
        u32::LE.trace("pixel_offset"),
    )
        .map(
            |(signature, file_size, reserved1, reserved2, pixel_offset)| BMPFileHeader {
                signature: *signature,
                file_size,
                reserved1,
                reserved2,
                pixel_offset,
            },
        )
        .trace("file_header");

    let width = u32::LE.trace("width").store();
    let width_ref = width.output();
    let height = u32::LE.trace("height").store();
    let height_ref = height.output();

    let info_header = (
        u32::LE.trace("header_size"),
        width,
        height,
        u16::LE.trace("color_planes"),
        u16::LE.trace("bits_per_pixel"),
        u32::LE.trace("compression"),
        u32::LE.trace("image_size"),
        u32::LE.trace("h_resolution"),
        u32::LE.trace("v_resolution"),
        u32::LE.trace("palette_colors"),
        u32::LE.trace("important_colors"),
    )
        .map(
            |(
                header_size,
                width,
                height,
                color_planes,
                bits_per_pixel,
                compression,
                image_size,
                h_resolution,
                v_resolution,
                palette_colors,
                important_colors,
            )| BMPInfoHeader {
                header_size,
                width,
                height,
                color_planes,
                bits_per_pixel,
                compression,
                image_size,
                h_resolution,
                v_resolution,
                palette_colors,
                important_colors,
            },
        )
        .trace("info_header");

    let pixel_data = u8::LE
        .repeat::<4>()
        .trace("pixel")
        .repeat_vec(width_ref.clone())
        .trace("row")
        .repeat_vec(height_ref.clone())
        .trace("pixel_data");

    (file_header, info_header, pixel_data)
        .map(|(file_header, info_header, pixel_data)| Bmp {
            file_header,
            info_header,
            pixel_data,
        })
        .trace("bmp")
}

fn main() {
    let mut parser = bmp_parser();

    let sample_bmp: &[u8] = &[
        // File header (14 bytes)
        0x42, 0x4D, // "BM"
        0x46, 0x00, 0x00, 0x00, // file_size = 70
        0x00, 0x00, // reserved1
        0x00, 0x00, // reserved2
        0x36, 0x00, 0x00, 0x00, // pixel_offset = 54
        // Info header (40 bytes)
        0x28, 0x00, 0x00, 0x00, // header_size = 40
        0x02, 0x00, 0x00, 0x00, // width = 2
        0x02, 0x00, 0x00, 0x00, // height = 2
        0x01, 0x00, // color_planes = 1
        0x20, 0x00, // bits_per_pixel = 32
        0x00, 0x00, 0x00, 0x00, // compression = 0
        0x10, 0x00, 0x00, 0x00, // image_size = 16
        0x00, 0x00, 0x00, 0x00, // h_resolution
        0x00, 0x00, 0x00, 0x00, // v_resolution
        0x00, 0x00, 0x00, 0x00, // palette_colors
        0x00, 0x00, 0x00, 0x00, // important_colors
        // Pixel data (16 bytes = 2 rows × 2 pixels × 4 bytes)
        // Row 1: blue pixel, green pixel
        0x00, 0x00, 0xFF, 0x00, 0x00, 0xFF, 0x00, 0x00,
        // Row 2: red pixel, white pixel
        0xFF, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0x00,
    ];

    println!("=== Parser structure ===");
    println!("{}", parser.spec());

    println!("\n=== Parse result ===");
    let mut input = sample_bmp;
    let (bmp, bytes_consumed) = parser.parse(&mut input).unwrap();
    println!("Bytes consumed: {}", bytes_consumed);
    println!("{:#?}", bmp);

    println!("\n=== Annotation tree ===");
    let mut input = sample_bmp;
    let (_bmp, annotation) = parser.annotate(&mut input).unwrap();
    println!("{:#?}", annotation);
}
