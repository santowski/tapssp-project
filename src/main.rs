// Stephen Santowski - Final Project
// some code inspired by textbook Programming Rust by Blandy, Orondorf, Tindell

use std::env;                                   // to obtain command line arguments
use std::fs::{File, OpenOptions};               // file opening & closing
use std::io::{BufRead, BufReader, Write};       // reading and writing to files
use flate2::Compression;                        // for compression - deflate
use flate2::write::ZlibEncoder;                 // deflate is found in zlib

struct ImageProperties {
    width: u32,
    height: u32,
    bit_depth: u8,
    color_type: u8,
    compression_method: u8,
    filter_method: u8,
    interlace_method: u8,
}

struct Arguments {

    source_file: String,
    output_file: String,
}

fn main() -> std::io::Result<()> {

    let args = parse_args();

    let image = set_image_properties();

    let mut encoded_png : Vec <u8> = Vec::new();

    encoded_png.extend(generate_signature());
    encoded_png.extend(generate_ihdr(&image));
    encoded_png.extend(generate_idat(&*args.source_file,image.width,image.color_type));
    encoded_png.extend(generate_iend());

    write_to_png(&*args.output_file, &encoded_png)?;
    Ok(())
}

fn parse_args() -> Arguments {

    let args : Vec<String> = env::args().skip(1).collect();

    if args.len() != 5 {
        print_usage();
        eprintln!("Error: wrong number of arguments: expected 5, got {}.", args.len());
        std::process::exit(1);
    }

    Arguments {

        source_file: args[3].clone(),
        output_file: args[4].clone(),

    }

}

fn set_image_properties() -> ImageProperties {

    let args : Vec<String> = env::args().skip(1).collect();

    ImageProperties {

        width: args[0].clone().parse::<u32>().unwrap(),
        height: args[1].clone().parse::<u32>().unwrap(),
        bit_depth: 8,
        color_type: args[2].clone().parse::<u8>().unwrap(),
        compression_method: 0,
        filter_method: 0,
        interlace_method: 0,

    }

}

fn print_usage() {

    eprintln!("Usage: pngencoder input output.png");

}

fn read_raw_image(path: &str) -> Vec<u8> {

    // convert raw image format into Vec<u8> of bytes

    let mut raw_image_output : Vec<u8> = Vec::new();

    let input = File::open(path).unwrap();
    let reader = BufReader::new(input);

    for line in reader.lines() {
        let mut line = line.unwrap();
        line = line.trim_end().parse().unwrap();
        let chunk : Vec<&str> = line.split(" ").collect();
        // println!("{:?}", chunk);
        let bytes: Vec<u8> = chunk.iter().map(|s| u8::from_str_radix(s, 2).unwrap()).collect();
        raw_image_output.extend(bytes);
    }

    // println!("{}",raw_image_output.len());



    raw_image_output

}

fn filter_image(raw_image : Vec<u8>, width : u32, color_type : u8) -> Vec<u8> {

    // We are using the filter type 0 = none.

    let mut filtered_image: Vec<u8> = Vec::new();

    let mut scanline_width = width;

    if color_type == 2 {
        scanline_width = width * 3;
    }

    let mut count = 0;

    for elem in raw_image {
        if count % scanline_width == 0 {
            filtered_image.push(0x00);
            // println!("{}", count);
        }
        filtered_image.push(elem);
        count += 1;
    }

   //intln!("{}",filtered_image.len());

    filtered_image

}

fn compress_image(filtered_image : Vec<u8>) -> Vec<u8> {

    // compress the filtered image data using the flate2 crate

    let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
    e.write_all(&filtered_image).unwrap();
    e.finish().unwrap()
}

fn generate_signature() -> Vec<u8> {

    // basic signature "magic number" for png files

    let signature : Vec<u8> = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

    signature

}

fn generate_ihdr(image : &ImageProperties) -> Vec<u8> {

    // creates the ihdr chunk

    let mut header : Vec<u8> = Vec::new();

    let length: Vec<u8> = vec![0, 0, 0, 13];
    header.extend(length);

    let chunk_type : Vec<u8> = vec![0x49, 0x48, 0x44, 0x52];
    header.extend(chunk_type);

    let width : Vec<u8> = image.width.to_be_bytes().to_vec();
    //println!("width: {:?}", width);
    header.extend(width);

    let height : Vec<u8> = image.height.to_be_bytes().to_vec();
    header.extend(height);

    let bit_depth : u8 = image.bit_depth;
    header.push(bit_depth);

    let color_type : u8 = image.color_type;
    header.push(color_type);

    let compression_method : u8 = image.compression_method;            // only 0 defined in standard
    header.push(compression_method);

    let filter_method :u8 = image.filter_method;                  // only 0 defined in standard
    header.push(filter_method);

    let interlace_method :u8 = image.interlace_method;               // only 0 or 1 defined in standard
    header.push(interlace_method);

    let crc_table = make_crc_table();

    let crc = crc(&header[4..],crc_table);
    let crc_vector = crc.to_be_bytes().to_vec();
    header.extend(crc_vector);

    header

}

fn generate_idat(path : &str, width : u32, color_type : u8) -> Vec<u8> {

    // creates an idat chunk

    let mut idat : Vec<u8> = Vec::new();

    let raw_image = read_raw_image(path);
    let filtered_image = filter_image(raw_image, width, color_type);
    let compressed_image = compress_image(filtered_image);

    // println!("{}",compressed_image.len());

    let n : u32 = compressed_image.len() as u32;
    let length: Vec<u8> = n.to_be_bytes().to_vec();
    // println!("{:?}",length);
    idat.extend(length);

    let chunk_type : Vec<u8> = vec![0x49, 0x44, 0x41, 0x54];
    idat.extend(chunk_type);

    idat.extend(compressed_image);

    let crc_table = make_crc_table();

    let crc = crc(&idat[4..],crc_table);
    let crc_vector = crc.to_be_bytes().to_vec();
    idat.extend(crc_vector);

    idat
}

fn make_crc_table() -> Vec<u32> {

    // This code was taken from the PNG standard and converted to Rust.

    let mut crc_table  = vec![0u32; 256];

    for n in 0..256 {
        let mut c = n as u32;
        for _k in 0..8 {
            if c & 1 != 0 {
                c = 0xedb88320 ^ (c >> 1);
            }
            else {
                c = c >> 1;
            }
        }
        crc_table[n] = c;
    }

    crc_table
}

fn update_crc(mut crc : u32, buf : &[u8], crc_table : Vec<u32>) -> u32 {

    let data = buf.to_vec();

    for n in 0..data.len() {
        crc = crc_table[((crc ^ data[n] as u32) & 0xff) as usize] ^ (crc >> 8);
    }

    crc

}

fn crc(buf : &[u8], crc_table : Vec<u32>) -> u32 {

    update_crc(0xFFFFFFFF, buf, crc_table) ^ 0xFFFFFFFF

}

fn generate_iend() -> Vec<u8> {

    let mut iend : Vec<u8> = Vec::new();

    let length: Vec<u8> = vec![0, 0, 0, 0];
    iend.extend(length);

    let chunk_type : Vec<u8> = vec![0x49, 0x45, 0x4E, 0x44];
    iend.extend(chunk_type);

    let crc_table = make_crc_table();

    let crc = crc(&iend[4..],crc_table);
    let crc_vector = crc.to_be_bytes().to_vec();
    iend.extend(crc_vector);

    iend

}

fn write_to_png(path: &str, bytes: &[u8]) -> Result<(), std::io::Error> {

    let mut file = OpenOptions::new().create(true).write(true).open(path)?;
    file.write_all(bytes)?;
    Ok(())
}
