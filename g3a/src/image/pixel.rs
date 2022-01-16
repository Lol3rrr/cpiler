#[derive(Debug, PartialEq, Clone)]
pub struct Pixel {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Pixel {
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }

    pub fn parse(data: &[u8; 2]) -> Pixel {
        let raw_red = data[0] >> 3;
        let raw_green = ((data[0] & 0b00000111) << 3) | ((data[1] & 0b11100000) >> 5);
        let raw_blue = data[1] & 0b00011111;

        Pixel {
            red: raw_red,
            green: raw_green,
            blue: raw_blue,
        }
    }

    pub fn serialize(&self) -> [u8; 2] {
        let mut out = [0, 0];

        out[0] = self.red << 3;
        out[0] |= 0b00000111 & (self.green >> 3);
        out[1] = 0b00011111 & self.blue;
        out[1] |= 0b11100000 & (self.green << 5);

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        let data = [0b00001000, 0b01100010];

        let expected = Pixel {
            red: 1,
            green: 3,
            blue: 2,
        };

        assert_eq!(expected, Pixel::parse(&data))
    }

    #[test]
    fn parse_serialize() {
        let starting_data = [0b00100110, 0b01100011];

        let parsed = Pixel::parse(&starting_data);

        assert_eq!(starting_data, parsed.serialize());
    }

    #[test]
    fn serialize_low_values() {
        let pixel = Pixel {
            red: 1,
            green: 3,
            blue: 2,
        };

        let expected: [u8; 2] = [0b00001000, 0b01100010];

        assert_eq!(expected, pixel.serialize());
    }
    #[test]
    fn serialize_high_values_1() {
        let pixel = Pixel {
            red: 0b00011111,
            green: 3,
            blue: 2,
        };

        let expected: [u8; 2] = [0b11111000, 0b01100010];

        assert_eq!(expected, pixel.serialize());
    }
    #[test]
    fn serialize_high_values_2() {
        let pixel = Pixel {
            red: 2,
            green: 0b00111111,
            blue: 2,
        };

        let expected: [u8; 2] = [0b00010111, 0b11100010];

        assert_eq!(expected, pixel.serialize());
    }
    #[test]
    fn serialize_high_values_3() {
        let pixel = Pixel {
            red: 2,
            green: 2,
            blue: 0b00011111,
        };

        let expected: [u8; 2] = [0b00010000, 0b01011111];

        assert_eq!(expected, pixel.serialize());
    }
}
