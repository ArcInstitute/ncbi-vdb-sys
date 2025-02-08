use std::io::{stdout, BufWriter, Write};

use anyhow::Result;
use clap::Parser;
use ncbi_vdb::{Segment, SraReader};

#[derive(Parser)]
pub struct Cli {
    pub path: String,

    /// The start of the range of records to dump
    ///
    /// Note: This is 1-based (zero will be replaced with 1)
    #[clap(short = 's', long, default_value = "1")]
    pub start: usize,

    /// The end of the range of records to dump
    ///
    /// Note: This is 1-based
    ///
    /// Note: If this is 0, the range will be from `start` to the end of the file
    #[clap(short = 'S', long, default_value = "0")]
    pub stop: usize,

    /// Include empty segments in the output (this will likely break a lot of tools)
    #[clap(short = 'e', long)]
    pub include_empty: bool,

    /// Include technical reads in the output
    #[clap(short = 't', long)]
    pub include_technical: bool,
}

fn write_segment<W: Write>(writer: &mut W, segment: Segment<'_>) -> Result<()> {
    writeln!(writer, "@{}.{}", segment.rid(), segment.sid())?;
    writer.write_all(segment.seq())?;
    writeln!(writer, "\n+")?;
    writer.write_all(segment.qual())?;
    writeln!(writer)?;
    Ok(())
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let reader = SraReader::new(&args.path)?;

    let start = if args.start == 0 {
        reader.start() as usize
    } else {
        args.start
    };
    let stop = if args.stop == 0 {
        reader.stop() as usize
    } else {
        args.stop
    };

    let writer = stdout();
    let mut writer = BufWriter::new(writer.lock());

    for record in reader.into_range_iter(start as i64, stop as u64)? {
        let record = record?;
        for segment in record.into_iter() {
            if !args.include_empty && segment.is_empty() {
                continue;
            }
            if !args.include_technical && segment.is_technical() {
                continue;
            }

            write_segment(&mut writer, segment)?;
        }
    }
    writer.flush()?;

    Ok(())
}
