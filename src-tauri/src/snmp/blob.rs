use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlobError {
    Truncated,
    MissingTerminator,
}

impl fmt::Display for BlobError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlobError::Truncated => write!(f, "record is truncated"),
            BlobError::MissingTerminator => write!(f, "missing 0xff terminator"),
        }
    }
}

impl std::error::Error for BlobError {}

pub fn parse_records(blob: &[u8]) -> Result<Vec<(u8, u32)>, BlobError> {
    let mut records = Vec::new();
    let mut rest = blob;

    loop {
        let Some((&id, tail)) = rest.split_first() else {
            return Err(BlobError::MissingTerminator);
        };

        if id == 0xff {
            return Ok(records);
        }

        let Some(&[_, _, a, b, c, d]) = tail.first_chunk::<6>() else {
            return Err(BlobError::Truncated);
        };

        records.push((id, u32::from_be_bytes([a, b, c, d])));
        rest = &tail[6..];
    }
}
