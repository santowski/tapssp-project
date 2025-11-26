# tapssp-project

[DEMO VIDEO](https://youtu.be/7r-RZSLSHyU)

## PNG Encoder in Rust

PNG is an image file format specified by [W3C](https://www.w3.org/TR/png-3/). The goal of this project is to build a basic PNG encoder in Rust.

## Dependencies

This code requires the flate2 crate for its deflate compression algorithm. 

## Run Instructions

After compiling via `cargo build`, you can type `cargo run` with the parameters below. Alternatively, after compilation you can also run:

```bash
pngencoder height width color_type input output.png
```


There are five arguments necessary to run pngencoder:
- height: the height of the image
- width: the width of the image
- color_type: 0 for grayscale or 2 for RGB
- input: the input raw source file
- output.png: the name of the output png file

## Source File Specifications

The source file should contain the bits of the image separated by the space character. For most images, it is preferable to use multiple lines instead of one, long line. Here is an example of a two pixel 8-bit RGB source code:

```bash
01100100 00010110 11001000
00001110 10001110 00110111
```

In this case, three bytes represent one pixel in the image. Alternatively, for grayscale, only one byte represents a pixel.

Please make sure that the number of bytes in the source file correctly matches the bytes needed for your specified color type and dimensions. For example, a grayscale image should have width x height bytes, whereas an RGB image should have width x height x 3 (one for each channel).

## How it works

The PNG file format consists of data chunks. There are three data chunks required to meet the minimum qualifications of the standard: the header, the data, and the end. These three chunks are implemented here. The remaining chunks are ancillary and contain (among other things) additional metadata. 

Within each chunk are three or four required sections:
- length: this is the length of the data section only. If there is no data section, then its value is 0.
- chunk type: this is a four byte name for the chunk, as specified in the W3C standard
- chunk data: this is the actual data. In the case of the data chunk, this is the compressed image data.
- crc: cyclic redundancy check. Four bytes calculated by XORing the chunk type and chunk type with the polynomial x^32 + x^26 + x^23 + x^22 + x^16 + x^12 + x^11 + x^10 + x^8 + x^7 + x^5 + x^4 + x^2 + x + 1 and then inverting the final remainder.

Before the image data is compressed, it can interlaced and filtered. Depending on the image type, filtering the image can benefit the compression step. In this implementation, we do not filter the image data or interlace the image. For most small images, this is sufficient. 
