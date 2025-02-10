mod core;
mod reader;
mod safe;

pub use reader::{RecordIter, RefRecord, Segment, SegmentIter, SegmentType, SraReader};
pub use safe::{Error, Result};
