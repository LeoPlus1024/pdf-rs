use crate::bytes::line_ending;
use crate::error::Result;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

pub trait Sequence {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
    /// Read a line data until encounter line delimiter
    fn read_line(&mut self) -> Result<Vec<u8>>;
    fn seek(&mut self, pos: u64) -> Result<u64>;
    fn size(&self) -> Result<u64>;
}

pub struct FileSequence {
    file: File,
    buf: Vec<u8>,
}

impl FileSequence {
    pub fn new(file: File) -> Self {
        let buf = Vec::new();
        Self { file, buf }
    }

    fn split_line_data(&mut self, index: usize) -> Vec<u8> {
        let mut buf = &mut self.buf;
        let line = buf.drain(0..index).collect::<Vec<u8>>();
        let len = buf.len();
        let mut crlf_num = 0;
        for i in 0..len {
            if !line_ending(buf[i]) {
                crlf_num = i;
                break;
            }
        }
        if crlf_num != 0 {
            buf.drain(0..crlf_num);
        }
        line
    }
}

impl Sequence for FileSequence {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let n = self.file.read(buf)?;
        // Due to read, the buffer is no longer valid
        self.buf.clear();
        Ok(n)
    }

    fn read_line(&mut self) -> Result<Vec<u8>> {
        let buf = &mut self.buf;
        let mut bytes = [0u8; 1024];
        let mut tmp = 0;
        loop {
            let n = self.file.read(&mut bytes)?;
            buf.extend_from_slice(&bytes[..n]);
            let len = buf.len();
            for i in tmp..len {
                if line_ending(buf[i]) {
                    let line_data = self.split_line_data(i);
                    return Ok(line_data);
                }
            }
            tmp = len;
        }
    }

    fn seek(&mut self, pos: u64) -> Result<u64> {
        let n = self.file.seek(SeekFrom::Start(pos))?;
        // Due to seek, the buffer is no longer valid
        self.buf.clear();
        Ok(n)
    }

    fn size(&self) -> Result<u64> {
        let n = self.file.metadata()?.len();
        Ok(n)
    }
}
