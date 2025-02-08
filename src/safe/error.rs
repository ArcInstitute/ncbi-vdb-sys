use crate::core::{klib::RCExplain, rc_t};

/// Type alias for `Result<T, Error>`
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("VDB error: {0}")]
    Vdb(String),
    #[error("Unexpected input VDB error: Cannot open input as DB or Table")]
    UnexpectedInputVdb,
    #[error("Unable to find column '{0}' in table")]
    UnexpectedColumnName(String),
    #[error("Provided range ({0}, {1}) is out of bounds for expected range ({2}, {3})")]
    CursorRangeError(usize, usize, usize, usize),
}

impl From<rc_t> for Error {
    fn from(rc: rc_t) -> Self {
        let details = rc_details(rc);
        Error::Vdb(details)
    }
}

pub fn rc_details(rc: rc_t) -> String {
    let mut buffer = [0 as std::os::raw::c_char; 1024];
    let mut num_writ = 0;
    unsafe {
        RCExplain(rc, buffer.as_mut_ptr(), buffer.len(), &mut num_writ);
    }
    let c_str = unsafe { std::ffi::CStr::from_ptr(buffer.as_ptr()) };
    c_str.to_string_lossy().into_owned()
}
