use crate::cache::FsCache;
use crate::{CodegenConfig, FsWriter, NoopWriter, Result, StdoutWriter, Writer};

pub struct CodegenRunner<'a> {
    pub config: &'a CodegenConfig,
    pub writer: Box<dyn Writer>,
}

impl<'a> CodegenRunner<'a> {
    pub fn new(config: &'a CodegenConfig) -> Self {
        Self {
            config,
            writer: Box::new(NoopWriter),
        }
    }

    // logger with different logging levels
    pub fn with_logger(mut self) -> Self {
        self
    }

    // noop for no-cache, in memory, etc...
    pub fn with_cache(mut self) -> Self {
        self
    }

    // fs, in memory, noop for check
    pub fn with_writer(mut self, writer: Box<dyn Writer>) -> Self {
        self.writer = writer;
        self
    }

    pub fn run(self) -> Result<()> {
        Ok(())
    }
}

struct RunInput<'a> {
    stdout: bool,
    check: bool,
    config: &'a CodegenConfig,
}

fn run(input: RunInput) -> Result<()> {
    let writer: Box<dyn Writer> = if input.stdout {
        Box::new(StdoutWriter::new())
    } else if input.check {
        Box::new(NoopWriter)
    } else {
        Box::new(FsWriter)
    };

    CodegenRunner::new(&input.config).with_writer(writer).run()
}
