use crate::{
    eactivity::EActivityError, image::ParseError as ImageError, localization::LocalizationError,
};

#[derive(Debug)]
pub enum FormatError {
    WrongControlBytes,
    InvalidShortName,
    InvalidInternalName,
}

#[derive(Debug)]
pub enum ParseError {
    WrongIdentifier,
    WrongFormat(FormatError),
    MismatchedChecksums,
    EActivity(EActivityError),
    Localization(LocalizationError),
    ImageError(ImageError),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WrongIdentifier => write!(f, "Wrong-Identifier"),
            Self::WrongFormat(e) => write!(f, "Wrong-Format: {:?}", e),
            Self::MismatchedChecksums => write!(f, "Checksums are not matching"),
            Self::EActivity(err) => write!(f, "{:?}", err),
            Self::Localization(err) => write!(f, "{:?}", err),
            Self::ImageError(err) => write!(f, "Image: {}", err),
        }
    }
}

impl From<EActivityError> for ParseError {
    fn from(other: EActivityError) -> Self {
        ParseError::EActivity(other)
    }
}
impl From<LocalizationError> for ParseError {
    fn from(other: LocalizationError) -> Self {
        ParseError::Localization(other)
    }
}
impl From<ImageError> for ParseError {
    fn from(other: ImageError) -> Self {
        ParseError::ImageError(other)
    }
}
