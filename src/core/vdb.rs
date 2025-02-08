use std::os::raw::{c_char, c_void};

use super::klib::{KDirectory, KNamelist};
use super::rc_t;

#[repr(C)]
pub struct VDBManager {
    _private: [u8; 0],
}

#[repr(C)]
pub struct VSchema {
    _private: [u8; 0],
}

#[repr(C)]
pub struct VDatabase {
    _private: [u8; 0],
}

#[repr(C)]
pub struct VTable {
    _private: [u8; 0],
}

#[repr(C)]
pub struct VCursor {
    _private: [u8; 0],
}

#[link(name = "ncbi-vdb")]
extern "C" {
    pub fn VDBManagerMakeRead(mgr: *mut *const VDBManager, dir: *mut KDirectory) -> rc_t;
    pub fn VDBManagerMakeSchema(mgr: *const VDBManager, schema: *mut *mut VSchema) -> rc_t;
    pub fn VDBManagerOpenDBRead(
        mgr: *const VDBManager,
        db: *mut *const VDatabase,
        schema: *mut VSchema,
        path: *const c_char,
        ...
    ) -> rc_t;
    pub fn VDBManagerOpenTableRead(
        mgr: *const VDBManager,
        tbl: *mut *const VTable,
        schema: *mut VSchema,
        path: *const c_char,
        ...
    ) -> rc_t;
    pub fn VDatabaseOpenTableRead(
        db: *const VDatabase,
        tbl: *mut *const VTable,
        name: *const c_char,
    ) -> rc_t;
    pub fn VTableCreateCachedCursorRead(
        tbl: *const VTable,
        cursor: *mut *const VCursor,
        capacity: usize,
    ) -> rc_t;
    pub fn VTableListCol(tbl: *const VTable, columns: *mut *mut KNamelist) -> rc_t;
    pub fn VCursorAddColumn(cursor: *const VCursor, idx: *mut u32, name: *const c_char) -> rc_t;
    pub fn VCursorOpen(cursor: *const VCursor) -> rc_t;
    pub fn VCursorIdRange(
        cursor: *const VCursor,
        idx: u32,
        first: *mut i64,
        count: *mut u64,
    ) -> rc_t;
    pub fn VCursorCellDataDirect(
        cursor: *const VCursor,
        row_id: i64,
        column_idx: u32,
        elem_bits: *mut u32,
        data: *mut *const c_void,
        bit_offset: *mut u32,
        row_len: *mut u32,
    ) -> rc_t;

    // Release functions
    pub fn VDBManagerRelease(self_: *const VDBManager) -> rc_t;
    pub fn VSchemaRelease(self_: *mut VSchema) -> rc_t;
    pub fn VDatabaseRelease(self_: *const VDatabase) -> rc_t;
    pub fn VTableRelease(self_: *const VTable) -> rc_t;
    pub fn VCursorRelease(self_: *const VCursor) -> rc_t;
}
