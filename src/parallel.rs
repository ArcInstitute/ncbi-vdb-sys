use crate::{RefRecord, Result};

pub const DEFAULT_BATCH_SIZE: usize = 1024;

pub trait ParallelReader {
    fn process_parallel<P: ParallelProcessor + Clone + 'static>(
        self,
        processor: P,
        num_threads: u64,
    ) -> Result<()>;
}

pub trait ParallelProcessor: Send + Clone {
    fn process_record(&mut self, record: RefRecord) -> Result<()>;

    #[allow(unused_variables)]
    fn on_batch_complete(&mut self) -> Result<()> {
        Ok(())
    }

    fn set_batch_size(&self) -> usize {
        DEFAULT_BATCH_SIZE
    }
}
