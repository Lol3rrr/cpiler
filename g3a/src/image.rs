use image::{io::Reader as ImageReader, DynamicImage, ImageBuffer, Rgb};

mod pixel;
pub use pixel::Pixel;

const IMAGE_ROWS: usize = 64;
const IMAGE_COLUMNS: usize = 92;
const IMAGE_SIZE: usize = IMAGE_ROWS * IMAGE_COLUMNS * 2;

#[derive(Debug)]
pub enum ParseError {
    InvalidSize(usize),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSize(other) => {
                write!(f, "Invalid Size, expected: {} got {}", IMAGE_SIZE, other)
            }
        }
    }
}

/// The custom Image Format used in the G3A Format
#[derive(Debug, Clone)]
pub struct Image {
    pixels: Vec<Vec<Pixel>>,
}

impl Image {
    /// Creates an Empty/Black Image
    pub fn empty() -> Self {
        let rows = vec![
            vec![
                Pixel {
                    red: 0,
                    green: 0,
                    blue: 0,
                };
                92
            ];
            64
        ];
        Self { pixels: rows }
    }

    /// Parses the raw Byte-Sequence as an Image
    pub fn parse(raw: &[u8]) -> Result<Self, ParseError> {
        let raw_len = raw.len();
        if raw_len != IMAGE_SIZE {
            return Err(ParseError::InvalidSize(raw_len));
        }

        let mut rows = Vec::with_capacity(IMAGE_ROWS);
        for y in 0..IMAGE_ROWS {
            let mut current_row = Vec::with_capacity(IMAGE_COLUMNS);
            for x in 0..IMAGE_COLUMNS {
                let index = (y * IMAGE_COLUMNS + x) * 2;
                let data = [raw[index], raw[index + 1]];

                current_row.push(Pixel::parse(&data));
            }

            rows.push(current_row);
        }

        Ok(Self { pixels: rows })
    }

    /// Serializes the Image into a Byte-Sequence to be stored
    /// in a G3A File
    pub fn serialize(&self) -> [u8; IMAGE_SIZE] {
        let mut result = [0; IMAGE_SIZE];

        for (y, row) in self.pixels.iter().enumerate() {
            for (x, pix) in row.iter().enumerate() {
                let first = (y * IMAGE_COLUMNS + x) * 2;
                let second = first + 1;

                let pix_data = pix.serialize();
                result[first] = pix_data[0];
                result[second] = pix_data[1];
            }
        }

        result
    }

    /// Reads in the File and then converts the Image into
    /// this custom Format used for the G3A File format
    pub fn from_file(path: &str) -> Option<Self> {
        let raw_img = match ImageReader::open(path) {
            Ok(s) => match s.decode() {
                Ok(i) => i,
                Err(_) => return None,
            },
            Err(_) => return None,
        };

        let img = match raw_img {
            DynamicImage::ImageRgb8(i) => i,
            _ => return None,
        };

        if img.width() != 92 || img.height() != 64 {
            return None;
        }

        let mut result_rows = Vec::with_capacity(64);
        for y in 0..64 {
            let mut current_row = Vec::with_capacity(92);
            for x in 0..92 {
                let pixel = img.get_pixel(x, y);

                current_row.push(Pixel::new(pixel[0], pixel[1], pixel[2]));
            }

            result_rows.push(current_row);
        }

        Some(Self {
            pixels: result_rows,
        })
    }

    /// Saves the Image into a File at the given Path
    pub fn save_to_file(&self, path: &str) -> Result<(), ()> {
        let width = 92;
        let height = 64;

        let mut imgbuf = ImageBuffer::new(width, height);

        for (y, row) in self.pixels.iter().enumerate() {
            for (x, pix) in row.iter().enumerate() {
                *(imgbuf.get_pixel_mut(x as u32, y as u32)) = Rgb([pix.red, pix.green, pix.blue]);
            }
        }

        match imgbuf.save(path) {
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        }
    }
}

impl Default for Image {
    fn default() -> Self {
        Self::empty()
    }
}
