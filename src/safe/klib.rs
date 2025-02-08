use crate::core::klib::{
    KDirectory, KDirectoryNativeDir, KDirectoryRelease, KNamelist, KNamelistRelease,
};
use crate::safe::{Error, Result};

// Safe wrapper over `KDirectory`
pub struct SafeKDirectory(pub(crate) *mut KDirectory);
impl Drop for SafeKDirectory {
    fn drop(&mut self) {
        unsafe { KDirectoryRelease(self.0) };
    }
}
impl SafeKDirectory {
    pub fn new() -> Result<Self> {
        let mut dir = std::ptr::null_mut();
        match unsafe { KDirectoryNativeDir(&mut dir) } {
            0 => Ok(SafeKDirectory(dir)),
            rc => Err(Error::from(rc)),
        }
    }
}

// Safe wrapper over `KNamelist`
pub struct SafeKNamelist(pub(crate) *mut KNamelist);
impl Drop for SafeKNamelist {
    fn drop(&mut self) {
        unsafe { KNamelistRelease(self.0) };
    }
}
