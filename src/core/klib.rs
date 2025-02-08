use std::os::raw::c_char;

use super::rc_t;

#[repr(C)]
pub struct KDirectory {
    _private: [u8; 0],
}

#[repr(C)]
pub struct KNamelist {
    _private: [u8; 0],
}

#[link(name = "ncbi-vdb")]
extern "C" {
    // Build functions
    #[link_name = "KDirectoryNativeDir_v1"]
    pub fn KDirectoryNativeDir(dir: *mut *mut KDirectory) -> rc_t;

    // Release functions
    #[link_name = "KDirectoryRelease_v1"]
    pub fn KDirectoryRelease(self_: *mut KDirectory) -> rc_t;
    pub fn KNamelistRelease(self_: *mut KNamelist) -> rc_t;

    // Utility functions
    pub fn KNamelistCount(list: *const KNamelist, count: *mut u32) -> rc_t;
    pub fn KNamelistGet(list: *const KNamelist, idx: u32, name: *mut *const c_char) -> rc_t;
    pub fn RCExplain(rc: rc_t, buffer: *mut c_char, bsize: usize, written: *mut usize) -> rc_t;
}
