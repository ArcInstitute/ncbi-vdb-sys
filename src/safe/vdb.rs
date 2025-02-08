use std::ffi::{CStr, CString};

use crate::{
    core::{
        klib::{KNamelistCount, KNamelistGet, KNamelistRelease},
        vdb::{
            VCursor, VCursorAddColumn, VCursorCellDataDirect, VCursorIdRange, VCursorOpen,
            VCursorRelease, VDBManager, VDBManagerMakeRead, VDBManagerMakeSchema,
            VDBManagerOpenDBRead, VDBManagerOpenTableRead, VDBManagerRelease, VDatabase,
            VDatabaseOpenTableRead, VDatabaseRelease, VSchema, VSchemaRelease, VTable,
            VTableCreateCachedCursorRead, VTableListCol, VTableRelease,
        },
    },
    safe::error::{Error, Result},
};

use super::klib::SafeKDirectory;

// Safe wrapper around `VDBManager`
pub struct SafeVDBManager(pub(crate) *const VDBManager);
impl Drop for SafeVDBManager {
    fn drop(&mut self) {
        unsafe { VDBManagerRelease(self.0) };
    }
}
impl SafeVDBManager {
    pub fn new(dir: &SafeKDirectory) -> Result<Self> {
        let mut mgr = std::ptr::null();
        match unsafe { VDBManagerMakeRead(&mut mgr, dir.0) } {
            0 => Ok(SafeVDBManager(mgr)),
            rc => Err(rc.into()),
        }
    }
    pub fn open_database(&self, schema: &SafeVSchema, path: &str) -> Result<Option<SafeVDatabase>> {
        let mut db = std::ptr::null();
        let path = CString::new(path).unwrap();

        match unsafe { VDBManagerOpenDBRead(self.0, &mut db, schema.0, path.as_ptr()) } {
            0 => Ok(Some(SafeVDatabase(db))),
            _ => Ok(None), // Not a database
        }
    }

    pub fn open_table(&self, schema: &SafeVSchema, path: &str) -> Result<Option<SafeVTable>> {
        let mut table = std::ptr::null();
        let path = CString::new(path).unwrap();

        match unsafe { VDBManagerOpenTableRead(self.0, &mut table, schema.0, path.as_ptr()) } {
            0 => Ok(Some(SafeVTable(table))),
            _ => Ok(None), // Not a table
        }
    }
}

// Safe wrapper around `VSchema`
pub struct SafeVSchema(pub *mut VSchema);
impl Drop for SafeVSchema {
    fn drop(&mut self) {
        unsafe { VSchemaRelease(self.0) };
    }
}
impl SafeVSchema {
    pub fn new(mgr: &SafeVDBManager) -> Result<Self> {
        let mut schema_ptr = std::ptr::null_mut();
        match unsafe { VDBManagerMakeSchema(mgr.0, &mut schema_ptr) } {
            0 => Ok(SafeVSchema(schema_ptr)),
            rc => Err(rc.into()),
        }
    }
}

// Safe wrapper around `VDatabase`
pub struct SafeVDatabase(pub *const VDatabase);
impl Drop for SafeVDatabase {
    fn drop(&mut self) {
        unsafe { VDatabaseRelease(self.0) };
    }
}
impl SafeVDatabase {
    pub fn open_table(&self, name: &str) -> Result<SafeVTable> {
        let mut table = std::ptr::null();
        let name = CString::new(name).unwrap();
        match unsafe { VDatabaseOpenTableRead(self.0, &mut table, name.as_ptr()) } {
            0 => Ok(SafeVTable(table)),
            rc => Err(rc.into()),
        }
    }
}

// Safe wrapper around `VTable`
pub struct SafeVTable(pub *const VTable);
impl Drop for SafeVTable {
    fn drop(&mut self) {
        unsafe { VTableRelease(self.0) };
    }
}
impl SafeVTable {
    pub fn new(path: &str) -> Result<Self> {
        let dir = SafeKDirectory::new()?;
        let mgr = SafeVDBManager::new(&dir)?;
        let schema = SafeVSchema::new(&mgr)?;
        Self::from_components(&mgr, &schema, path)
    }

    pub fn from_components(
        mgr: &SafeVDBManager,
        schema: &SafeVSchema,
        sra_file: &str,
    ) -> Result<Self> {
        match mgr.open_database(schema, sra_file) {
            Ok(Some(db)) => db.open_table("SEQUENCE"),
            Ok(None) => match mgr.open_table(schema, sra_file) {
                Ok(Some(table)) => Ok(table),
                Ok(None) => Err(Error::UnexpectedInputVdb), // Failed to open input as either database or table
                Err(e) => Err(e),                           // VDBManagerOpenTableRead failed
            },
            Err(e) => Err(e), // VDatabaseOpenDBFailed failed
        }
    }

    pub fn contains_column(&self, col_name: &str) -> Result<bool> {
        let mut columns = std::ptr::null_mut();
        match unsafe { VTableListCol(self.0, &mut columns) } {
            0 => {
                // continue
            }
            rc => {
                return Err(rc.into());
            }
        }

        let mut count = 0;
        match unsafe { KNamelistCount(columns, &mut count) } {
            0 => {
                // continue
            }
            rc => {
                unsafe { KNamelistRelease(columns) };
                return Err(rc.into());
            }
        }

        let mut present = false;
        for i in 0..count {
            let mut name_ptr = std::ptr::null();
            match unsafe { KNamelistGet(columns, i, &mut name_ptr) } {
                0 => {
                    // continue
                }
                rc => {
                    unsafe { KNamelistRelease(columns) };
                    return Err(rc.into());
                }
            }
            let name = unsafe { CStr::from_ptr(name_ptr) }.to_str().unwrap();
            if name == col_name {
                present = true;
                break;
            }
        }

        unsafe { KNamelistRelease(columns) };
        Ok(present)
    }

    pub fn new_fastq_cursor(self, buffer_size: usize) -> Result<FastqCursor> {
        FastqCursor::new(self, buffer_size)
    }
}

// Safe wrapper around `VCursor`
// pub struct SafeVCursor(pub *const VCursor);
pub struct SafeVCursor {
    table: SafeVTable,
    cursor: *const VCursor,
}
impl Drop for SafeVCursor {
    fn drop(&mut self) {
        unsafe {
            VTableRelease(self.table.0);
            VCursorRelease(self.cursor);
        };
    }
}
impl SafeVCursor {
    pub fn new(table: SafeVTable, buffer_size: usize) -> Result<Self> {
        let mut cursor = std::ptr::null();
        match unsafe { VTableCreateCachedCursorRead(table.0, &mut cursor, buffer_size) } {
            0 => Ok(Self { table, cursor }),
            rc => Err(rc.into()),
        }
    }
    pub fn get_range(&self) -> Result<(i64, u64)> {
        let mut first_row_id = 0;
        let mut row_count = 0;
        match unsafe { VCursorIdRange(self.cursor, 0, &mut first_row_id, &mut row_count) } {
            0 => Ok((first_row_id, row_count)),
            rc => Err(rc.into()),
        }
    }
    pub fn as_ptr(&self) -> *const VCursor {
        self.cursor
    }

    pub fn add_column(&self, col_name: &str) -> Result<u32> {
        let mut idx = 0;
        let col_name = CString::new(col_name).unwrap();
        match unsafe { VCursorAddColumn(self.cursor, &mut idx, col_name.as_ptr()) } {
            0 => Ok(idx),
            rc => Err(rc.into()),
        }
    }

    pub fn add_column_spec(&self, col_name: &str, spec: &str) -> Result<u32> {
        match self.table.contains_column(col_name) {
            Ok(true) => match self.add_column(spec) {
                Ok(idx) => Ok(idx),
                Err(e) => Err(e),
            },
            Ok(false) => Err(Error::UnexpectedColumnName(col_name.to_string())),
            Err(e) => Err(e),
        }
    }

    pub fn open(&self) -> Result<()> {
        match unsafe { VCursorOpen(self.cursor) } {
            0 => Ok(()),
            rc => Err(rc.into()),
        }
    }
}

/// Column indices used when reading data from a table
#[derive(Debug, Default)]
pub struct FastqIndices {
    pub seq: u32,
    pub qual: u32,
    pub read_start: u32,
    pub read_len: u32,
    pub read_type: u32,
}

pub struct FastqCursor {
    inner: SafeVCursor,
    pub indices: FastqIndices,
}
impl FastqCursor {
    fn new(table: SafeVTable, buffer_size: usize) -> Result<Self> {
        // Create a new cursor
        let cursor = SafeVCursor::new(table, buffer_size)?;

        // Add relevant columns
        let seq_idx = cursor.add_column("READ")?;
        let qual_idx = cursor.add_column_spec("QUALITY", "(INSDC:quality:text:phred_33)QUALITY")?;
        let read_start_idx =
            cursor.add_column_spec("READ_START", "(INSDC:coord:zero)READ_START")?;
        let read_len_idx = cursor.add_column_spec("READ_LEN", "(INSDC:coord:len)READ_LEN")?;
        let read_type_idx =
            cursor.add_column_spec("READ_TYPE", "(INSDC:SRA:xread_type)READ_TYPE")?;

        // Create the FastqIndices struct
        let indices = FastqIndices {
            seq: seq_idx,
            qual: qual_idx,
            read_start: read_start_idx,
            read_len: read_len_idx,
            read_type: read_type_idx,
        };

        // Open the cursor
        cursor.open()?;

        // Return the FastqCursor
        Ok(Self {
            inner: cursor,
            indices,
        })
    }

    pub fn get_range(&self) -> Result<(i64, u64)> {
        self.inner.get_range()
    }

    pub fn get_read(&self, row_id: i64) -> Result<&[u8]> {
        let mut row_len = 0;
        let mut seq_data = std::ptr::null();
        match unsafe {
            VCursorCellDataDirect(
                self.inner.as_ptr(),
                row_id,
                self.indices.seq,
                std::ptr::null_mut(),
                &mut seq_data as *mut *const _,
                std::ptr::null_mut(),
                &mut row_len,
            )
        } {
            0 => Ok(unsafe { std::slice::from_raw_parts(seq_data as *const u8, row_len as usize) }),
            rc => Err(rc.into()),
        }
    }

    pub fn get_qual(&self, row_id: i64) -> Result<&[u8]> {
        let mut row_len = 0;
        let mut qual_data = std::ptr::null();
        match unsafe {
            VCursorCellDataDirect(
                self.inner.as_ptr(),
                row_id,
                self.indices.qual,
                std::ptr::null_mut(),
                &mut qual_data as *mut *const _,
                std::ptr::null_mut(),
                &mut row_len,
            )
        } {
            0 => {
                Ok(unsafe { std::slice::from_raw_parts(qual_data as *const u8, row_len as usize) })
            }
            rc => Err(rc.into()),
        }
    }

    pub fn get_read_starts(&self, row_id: i64) -> Result<&[u32]> {
        let mut num_reads = 0;
        let mut read_starts = std::ptr::null();
        match unsafe {
            VCursorCellDataDirect(
                self.inner.as_ptr(),
                row_id,
                self.indices.read_start,
                std::ptr::null_mut(),
                &mut read_starts as *mut *const _,
                std::ptr::null_mut(),
                &mut num_reads,
            )
        } {
            0 => Ok(unsafe {
                std::slice::from_raw_parts(read_starts as *const u32, num_reads as usize)
            }),
            rc => Err(rc.into()),
        }
    }

    pub fn get_read_lens(&self, row_id: i64) -> Result<&[u32]> {
        let mut num_reads = 0;
        let mut read_lens = std::ptr::null();
        match unsafe {
            VCursorCellDataDirect(
                self.inner.as_ptr(),
                row_id,
                self.indices.read_len,
                std::ptr::null_mut(),
                &mut read_lens as *mut *const _,
                std::ptr::null_mut(),
                &mut num_reads,
            )
        } {
            0 => Ok(unsafe {
                std::slice::from_raw_parts(read_lens as *const u32, num_reads as usize)
            }),
            rc => Err(rc.into()),
        }
    }

    pub fn get_read_types(&self, row_id: i64) -> Result<&[u8]> {
        let mut num_reads = 0;
        let mut read_types = std::ptr::null();
        match unsafe {
            VCursorCellDataDirect(
                self.inner.as_ptr(),
                row_id,
                self.indices.read_type,
                std::ptr::null_mut(),
                &mut read_types as *mut *const _,
                std::ptr::null_mut(),
                &mut num_reads,
            )
        } {
            0 => Ok(unsafe {
                std::slice::from_raw_parts(read_types as *const u8, num_reads as usize)
            }),
            rc => Err(rc.into()),
        }
    }
}
