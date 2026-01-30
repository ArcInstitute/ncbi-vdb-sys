use crate::safe::vdb::{FastqCursor, SafeVTable};
use crate::{Error, Result};

pub const BUFFER_SIZE: usize = 1024 * 1024; // 1MB buffer

pub struct SraReader {
    pub cursor: FastqCursor,
    start: i64,
    stop: u64,
    pos: i64,
}
impl SraReader {
    pub fn new(path: &str) -> Result<Self> {
        let table = SafeVTable::new(path)?;
        let cursor = table.new_fastq_cursor(BUFFER_SIZE)?;
        let (start, stop) = cursor.get_range()?;
        Ok(Self {
            cursor,
            start,
            stop,
            pos: start,
        })
    }
    pub fn with_capacity(path: &str, capacity: usize) -> Result<Self> {
        let table = SafeVTable::new(path)?;
        let cursor = table.new_fastq_cursor(capacity)?;
        let (start, stop) = cursor.get_range()?;
        Ok(Self {
            cursor,
            start,
            stop,
            pos: start,
        })
    }
    pub fn start(&self) -> i64 {
        self.start
    }
    pub fn stop(&self) -> u64 {
        self.stop
    }
    pub fn pos(&self) -> i64 {
        self.pos
    }
    pub fn get_record(&self, row_id: i64) -> Result<RefRecord<'_>> {
        let rid = row_id as usize;
        let seq = self.cursor.get_read(row_id)?;
        let qual = self.cursor.get_qual(row_id)?;
        let read_starts = self.cursor.get_read_starts(row_id)?;
        let read_lens = self.cursor.get_read_lens(row_id)?;
        let read_types = self.cursor.get_read_types(row_id)?;
        Ok(RefRecord {
            rid,
            seq,
            qual,
            read_starts,
            read_lens,
            read_types,
        })
    }
    pub fn into_iter(&self) -> RecordIter<'_> {
        RecordIter::new(self)
    }
    pub fn into_range_iter(&self, start: i64, stop: u64) -> Result<RecordIter<'_>> {
        if start < self.start() || stop > self.stop() || start > stop as i64 {
            return Err(Error::CursorRangeError(
                start as usize,
                stop as usize,
                self.start() as usize,
                self.stop() as usize,
            ));
        }
        Ok(RecordIter::new_range(self, start, stop))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SegmentType {
    /// Technical read (0)
    Technical,
    /// Biological read (1)
    Biological,
}
impl From<u8> for SegmentType {
    fn from(ty: u8) -> Self {
        match ty {
            0 => SegmentType::Technical,
            _ => SegmentType::Biological,
        }
    }
}
impl From<SegmentType> for u8 {
    fn from(ty: SegmentType) -> Self {
        match ty {
            SegmentType::Technical => 0,
            SegmentType::Biological => 1,
        }
    }
}

pub struct RefRecord<'a> {
    /// Row ID (Spot ID)
    pub rid: usize,
    /// Spot sequence
    pub seq: &'a [u8],
    /// Spot quality
    pub qual: &'a [u8],
    /// Read segment start positions
    pub read_starts: &'a [u32],
    /// Read segment lengths
    pub read_lens: &'a [u32],
    /// Read segment types
    pub read_types: &'a [u8],
}
impl<'a> RefRecord<'a> {
    #[allow(clippy::should_implement_trait)]
    pub fn into_iter(self) -> SegmentIter<'a> {
        SegmentIter::new(self)
    }

    pub fn get_segment(&self, index: usize) -> Option<Segment<'a>> {
        if index >= self.read_starts.len() {
            return None;
        }
        let start = self.read_starts[index] as usize;
        let len = self.read_lens[index] as usize;
        Some(Segment {
            rid: self.rid,
            sid: index,
            seq: &self.seq[start..start + len],
            qual: &self.qual[start..start + len],
            ty: self.read_types[index].into(),
        })
    }
}

pub struct Segment<'a> {
    /// Row ID (Spot ID)
    rid: usize,
    /// Segment ID
    sid: usize,
    /// Segment sequence
    seq: &'a [u8],
    /// Segment quality
    qual: &'a [u8],
    /// Segment type
    ty: SegmentType,
}
impl Segment<'_> {
    pub fn rid(&self) -> usize {
        self.rid
    }
    pub fn sid(&self) -> usize {
        self.sid
    }
    pub fn seq(&self) -> &[u8] {
        self.seq
    }
    pub fn qual(&self) -> &[u8] {
        self.qual
    }
    pub fn ty(&self) -> SegmentType {
        self.ty
    }
    pub fn len(&self) -> usize {
        self.seq.len()
    }
    pub fn is_technical(&self) -> bool {
        self.ty == SegmentType::Technical
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub struct RecordIter<'a> {
    reader: &'a SraReader,
    pos: i64,
    end: u64,
}
impl<'a> RecordIter<'a> {
    pub fn new(reader: &'a SraReader) -> Self {
        Self {
            reader,
            pos: reader.start(),
            end: reader.stop(),
        }
    }
    pub fn new_range(reader: &'a SraReader, start: i64, stop: u64) -> Self {
        Self {
            reader,
            pos: start,
            end: stop,
        }
    }
}
impl<'a> Iterator for RecordIter<'a> {
    type Item = Result<RefRecord<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos > self.end as i64 {
            return None;
        }
        match self.reader.get_record(self.pos) {
            Ok(record) => {
                self.pos += 1;
                Some(Ok(record))
            }
            Err(rc) => {
                self.pos += 1;
                Some(Err(rc))
            }
        }
    }
}

pub struct SegmentIter<'a> {
    record: RefRecord<'a>,
    pos: usize,
}
impl<'a> SegmentIter<'a> {
    pub fn new(record: RefRecord<'a>) -> Self {
        Self { pos: 0, record }
    }
}
impl<'a> Iterator for SegmentIter<'a> {
    type Item = Segment<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let segment = self.record.get_segment(self.pos)?;
        self.pos += 1;
        Some(segment)
    }
}
