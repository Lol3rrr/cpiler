use crate::{eactivity, image, localization, ParseError};
use crate::{error::FormatError, util};

use std::convert::TryInto;

const HEADER_IDENTIFIER: [u8; 14] = [
    0xAA, 0xAC, 0xBD, 0xAF, 0x90, 0x88, 0x9A, 0x8D, 0xD3, 0xFF, 0xFE, 0xFF, 0xFE, 0xFF,
];
const CHECKSUM_1: usize = 0x0020;
const EXECUTABLE_SIZE: usize = 0x002E;
const SHORT_NAME: usize = 0x0040;
const INTERNAL_NAME: usize = 0x0060;
const UNSELECTED_ICON: usize = 0x1000;
const SELECTED_ICON: usize = 0x4000;
const EXECUTABLE_SECTION: usize = 0x7000;

/// A single G3A-File
#[derive(Debug)]
pub struct File {
    /// The Internal-Name of the Application
    pub internal_name: String,
    /// The Short-Name of the Application
    pub short_name: String,
    /// The total Size of the File
    pub file_size: u32,
    /// The Selected Image that is displayed when the
    /// Add-In is being selected on the MainMenu
    pub selected_image: image::Image,
    /// The Image that is displayed on the MainMenu
    /// if it is not selected
    pub unselected_image: image::Image,
    /// The Code portion of the File
    pub executable_code: Vec<u8>,
    /// The localization Options for the Application
    /// containing the Names in different Languages
    pub localized: localization::Localized,
    /// The EActivity Settings regarding this Add-In
    pub eactivity: eactivity::EActivity,
}

// References
// https://prizm.cemetech.net/index.php/G3A_File_Format
// https://www.omnimaga.org/casio-calculator-programming-news-and-support/casio-prizm-already-for-sale/msg157437/#msg157437
// https://gitlab.com/taricorp/mkg3a
impl File {
    /// Tries to parse the given Byte-Sequence into a G3A-File
    pub fn parse(content: &[u8]) -> Result<File, ParseError> {
        let identifier = &content[0..14];
        if identifier != HEADER_IDENTIFIER {
            return Err(ParseError::WrongIdentifier);
        }

        if content[0x000f] != 0xfe {
            return Err(ParseError::WrongFormat(FormatError::WrongControlBytes));
        }

        let raw_file_size = [
            content[0x0010] ^ 0xff,
            content[0x0011] ^ 0xff,
            content[0x0012] ^ 0xff,
            content[0x0013] ^ 0xff,
        ];

        let file_size = u32::from_be_bytes(raw_file_size);

        let raw_checksum = &content[CHECKSUM_1..CHECKSUM_1 + 4];
        let checksum = u32::from_be_bytes(raw_checksum.try_into().unwrap());

        if content[0x0024..0x0026] != [0x01, 0x01] {
            return Err(ParseError::WrongFormat(FormatError::WrongControlBytes));
        }

        let raw_executable_size = &content[EXECUTABLE_SIZE..EXECUTABLE_SIZE + 4];
        let executable_size = u32::from_be_bytes(raw_executable_size.try_into().unwrap());

        let raw_short_name = &content[SHORT_NAME..SHORT_NAME + 0x1c];
        let short_name = match String::from_utf8(raw_short_name.to_vec()) {
            Ok(s) => s,
            Err(_) => return Err(ParseError::WrongFormat(FormatError::InvalidShortName)),
        };

        let raw_internal_name = &content[INTERNAL_NAME..INTERNAL_NAME + 0x0b];
        let internal_name = match String::from_utf8(raw_internal_name.to_vec()) {
            Ok(s) => s,
            Err(_) => return Err(ParseError::WrongFormat(FormatError::InvalidInternalName)),
        };

        let raw_icon_unselected = &content[UNSELECTED_ICON..UNSELECTED_ICON + 0x2e00];
        let icon_unselected = image::Image::parse(raw_icon_unselected)?;

        let raw_icon_selected = &content[SELECTED_ICON..SELECTED_ICON + 0x2e00];
        let icon_selected = image::Image::parse(raw_icon_selected)?;

        let executable_end: usize = EXECUTABLE_SECTION + executable_size as usize;
        let raw_executable_section = &content[EXECUTABLE_SECTION..executable_end];

        let raw_checksum_copy = &content[executable_end..executable_end + 4];
        let checksum_copy = u32::from_be_bytes(raw_checksum_copy.try_into().unwrap());

        if checksum != checksum_copy {
            return Err(ParseError::MismatchedChecksums);
        }

        let localized = localization::Localized::parse(content)?;
        let parsed_eactivity = eactivity::EActivity::parse(content)?;

        Ok(File {
            internal_name,
            short_name,
            file_size,
            selected_image: icon_selected,
            unselected_image: icon_unselected,
            executable_code: raw_executable_section.to_vec(),
            localized,
            eactivity: parsed_eactivity,
        })
    }

    /// Serializes the File into a valid Byte-Sequence
    pub fn serialize(&self, filename: &str) -> Vec<u8> {
        let mut result = vec![0; 0x7000];

        result[0..0x000e].copy_from_slice(&HEADER_IDENTIFIER);

        // **
        // General Header stuff
        // **
        let file_size = self.file_size.to_be_bytes();
        // 0x000E
        result[0x000e] = (file_size[3] ^ 0xff).wrapping_sub(0x41);
        // 0x000F
        result[0x000f] = 0xfe;
        // 0x0010
        result[0x0010] = file_size[0] ^ 0xff;
        result[0x0011] = file_size[1] ^ 0xff;
        result[0x0012] = file_size[2] ^ 0xff;
        result[0x0013] = file_size[3] ^ 0xff;
        // 0x0014
        result[0x0014] = (file_size[3] ^ 0xff).wrapping_sub(0xb8);
        // 0x0016
        // TODO
        result[0x0016..0x0016 + 4].copy_from_slice(&0_u32.to_be_bytes());
        // 0x0024
        result[0x0024] = 0x01;
        result[0x0025] = 0x01;
        // 0x002e
        result[0x002e..0x002e + 4]
            .copy_from_slice(&(self.executable_code.len() as u32).to_be_bytes());
        // 0x0040
        util::write_string(&mut result[0x0040..0x005c], &self.short_name);
        // 0x005c
        result[0x005c..0x005c + 4].copy_from_slice(&self.file_size.to_be_bytes());
        // 0x0060
        util::write_string(&mut result[0x0060..0x006b], &self.internal_name);

        // **
        // Unselected image
        // **
        result[0x1000..=0x3dff].copy_from_slice(&self.unselected_image.serialize());

        // **
        // Selected image
        // **
        result[0x4000..=0x6dff].copy_from_slice(&self.selected_image.serialize());

        // **
        // Localization Stuff
        // **
        result[0x006b..0x014a].copy_from_slice(&self.localized.serialize());

        // **
        // EActivity
        // **
        result[0x0170..0x0590].copy_from_slice(&self.eactivity.serialize());

        util::write_string(&mut result[0x0ebc..0x1000], filename);

        // **
        // Code Block
        // **
        result.extend_from_slice(&self.executable_code);

        // Checksum at the end
        let binary_checksum = util::checksum(&self.executable_code);
        let header_checksum = util::checksum(&result[..0x7000]);
        let checksum = binary_checksum.wrapping_add(header_checksum);
        let chechsum_bytes = checksum.to_be_bytes();
        result[0x0020..0x0024].copy_from_slice(&chechsum_bytes);
        result.extend_from_slice(&chechsum_bytes);

        result
    }
}
