# ncbi-vdb

This crate provides FFIs for the ncbi-vdb C-library.

It provides a safe and structured way to interact with the SRA file format (Sequence Read Archive).

The library is designed to be used primarily with [`xsra`](https://github.com/arcinstitute/xsra), but is available as a standalone crate for other projects.

## Usage

The library is designed around accessing `Record`s and `Segment`s from an SRA archive.
A `Record` is a collection of `Segment`s that represent a single read (or spot) from the archive.
The library is designed around `Record` and `Segment` Iterators.

```rust

use ncbi_vdb::{SraReader, Result};

fn main() -> Result<()> {
    let path = "path/to/sra/file.sra";
    let reader = SraReader::new(path)?;

    // Iterate over all records in the archive
    for record in reader.into_iter() {

        // Handle any errors that occur at the record level
        let record = record?;

        // Iterate over all segments in the record
        for segment in record {
            // Do something with the segment
        }
    }
    Ok(())
}
```

Each `Segment` contains the sequence data and quality scores for a single read segment.
You can also access the `SegmentType` to determine if it is a biological read or a technical read.

For a simple example of how you can use the library, see the `examples` directory.
I've implemented a basic `fastq-dump` style tool that will extract all records from an SRA archive and write them to stdout.
This also supports specifying a range of records to extract.

```bash
git clone https://github.com/noamteyssier/ncbi-vdb
cd ncbi-vdb
cargo build --release --examples dump

# Run the example
target/release/examples/dump <path/to/sra/file.sra>
```

## Limitations

This library is not meant to be a complete feature-for-feature replacement for the `ncbi-vdb` C-library.
It was written for a specific use-case and may not support all possible SRA archive layouts.

Note that this will build the `ncbi-vdb` C-library as a static library and link against it.
This may make your resulting binary system-specific and may not work on all systems (will be limited to what ncbi-vdb can be built on).

Note: Interfacing with the underlying C-library is unsafe. This library is not fully tested against all possible edge-cases so while it should be safe to use, there may be bugs.

## Contributing

If you have a specific use-case that is not supported by this library, please open an issue or a pull request.
