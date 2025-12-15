use std::cmp::min;
use crate::bytes::{count_leading_line_endings, line_ending};
use crate::error::Result;
use crate::error::error_kind::{EOF, SEEK_EXEED_MAX_SIZE};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

pub trait Sequence {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
    /// Read a line data until encounter line delimiter
    fn read_line(&mut self) -> Result<Vec<u8>>;
    /// Read a line data as string until encounter line delimiter
    fn read_line_str(&mut self) -> Result<String>;
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
        let buf = &mut self.buf;
        let line = buf.drain(0..index).collect::<Vec<u8>>();
        buf.len();
        let crlf_num = count_leading_line_endings(buf);
        if crlf_num != 0 {
            buf.drain(0..crlf_num as usize);
        }
        line
    }
}

impl Sequence for FileSequence {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if !self.buf.is_empty() {
            let len = self.buf.len();
            let n = min(len, buf.len());
            buf[0..n].copy_from_slice(&self.buf[0..n]);
            self.buf.drain(0..n);
            return Ok(n);
        }
        let n = self.file.read(buf)?;
        Ok(n)
    }

    fn read_line(&mut self) -> Result<Vec<u8>> {
        let buf = &mut self.buf;
        let mut bytes = [0u8; 1024];
        let mut tmp = 0;
        loop {
            let len = buf.len();
            for i in tmp..len {
                if line_ending(buf[i]) {
                    let line_data = self.split_line_data(i);
                    return Ok(line_data);
                }
            }
            tmp = len;
            let n = self.file.read(&mut bytes)?;
            if n == 0 {
                return Err(EOF.into());
            }
            let offset = if len == 0 {
                count_leading_line_endings(&bytes)
            }else {
                0u64
            } as usize;
            buf.extend_from_slice(&bytes[offset..n]);
        }
    }

    fn read_line_str(&mut self) -> Result<String> {
        let buf = self.read_line()?;
        let text = String::from_utf8(buf)?;
        Ok(text)
    }


    fn seek(&mut self, pos: u64) -> Result<u64> {
        if self.size()? < pos {
            return Err(SEEK_EXEED_MAX_SIZE.into());
        }
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
