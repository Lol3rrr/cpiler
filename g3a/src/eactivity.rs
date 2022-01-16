use crate::util::write_string;

#[derive(Debug)]
pub enum Language {
    English,
    Spanish,
    German,
    French,
    Portuguese,
    Chinese,
}

#[derive(Debug)]
pub enum EActivityError {
    MalformedLanguage(Language),
    TooSmall,
}

/// The EActivity options for the Add-In
#[derive(Debug)]
pub struct EActivity {
    english: String,
    spanish: String,
    german: String,
    french: String,
    portuguese: String,
    chinese: String,
    icon: [u8; 0x0300],
}

fn parse_language(part: Vec<u8>, lang: Language) -> Result<String, EActivityError> {
    match String::from_utf8(part) {
        Ok(s) => Ok(s),
        Err(_) => Err(EActivityError::MalformedLanguage(lang)),
    }
}

impl EActivity {
    /// Creates an empty EActivity
    pub fn empty() -> Self {
        Self {
            english: String::new(),
            spanish: String::new(),
            german: String::new(),
            french: String::new(),
            portuguese: String::new(),
            chinese: String::new(),
            icon: [0; 0x0300],
        }
    }

    /// Parses the EActivity from a G3A File, accepts
    /// the entire File as input
    pub fn parse(content: &[u8]) -> Result<Self, EActivityError> {
        if content.len() < 0x0590 {
            return Err(EActivityError::TooSmall);
        }

        let raw_english = &content[0x0170..0x0194];
        let raw_spanish = &content[0x0194..0x01b8];
        let raw_german = &content[0x01b8..0x01dc];
        let raw_french = &content[0x01dc..0x0200];
        let raw_portuguese = &content[0x0200..0x0224];
        let raw_chinese = &content[0x0224..0x0248];

        let english = parse_language(raw_english.to_vec(), Language::English)?;
        let spanish = parse_language(raw_spanish.to_vec(), Language::Spanish)?;
        let german = parse_language(raw_german.to_vec(), Language::German)?;
        let french = parse_language(raw_french.to_vec(), Language::French)?;
        let portuguese = parse_language(raw_portuguese.to_vec(), Language::Portuguese)?;
        let chinese = parse_language(raw_chinese.to_vec(), Language::Chinese)?;

        let raw_icon = &content[0x0290..0x0590];
        let mut icon = [0; 0x0300];
        icon.copy_from_slice(raw_icon);

        Ok(Self {
            english,
            spanish,
            german,
            french,
            portuguese,
            chinese,
            icon,
        })
    }

    /// Serializes the current EActivity into a valid Byte-Sequence
    pub fn serialize(&self) -> [u8; 0x420] {
        let mut result = [0; 0x420];

        write_string(&mut result[0x0..], &self.english);
        write_string(&mut result[0x24..], &self.spanish);
        write_string(&mut result[0x48..], &self.german);
        write_string(&mut result[0x6c..], &self.french);
        write_string(&mut result[0x90..], &self.portuguese);
        write_string(&mut result[0xb4..], &self.chinese);

        // Reserved, filled with english in meantime
        write_string(&mut result[0xd8..], &self.english);
        write_string(&mut result[0xfc..], &self.english);

        result[0x120..].copy_from_slice(&self.icon);

        result
    }
}
