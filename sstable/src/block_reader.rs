use std::io;
use std::ops::Range;

pub struct BlockReader<'a> {
    buffer: Vec<u8>,
    reader: Box<dyn io::Read + 'a>,
    offset: usize,
}

#[inline]
fn read_u32(read: &mut dyn io::Read) -> io::Result<u32> {
    let mut buf = [0u8; 4];
    read.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

impl<'a> BlockReader<'a> {
    pub fn new(reader: Box<dyn io::Read + 'a>) -> BlockReader<'a> {
        BlockReader {
            buffer: Vec::new(),
            reader,
            offset: 0,
        }
    }

    pub fn deserialize_u64(&mut self) -> u64 {
        let (num_bytes, val) = super::vint::deserialize_read(self.buffer());
        self.advance(num_bytes);
        val
    }

    #[inline(always)]
    pub fn buffer_from_to(&self, range: Range<usize>) -> &[u8] {
        &self.buffer[range]
    }

    pub fn read_block(&mut self) -> io::Result<bool> {
        self.offset = 0;
        let block_len_res = read_u32(self.reader.as_mut());
        if let Err(err) = &block_len_res {
            if err.kind() == io::ErrorKind::UnexpectedEof {
                return Ok(false);
            }
        }
        let block_len = block_len_res?;
        if block_len == 0u32 {
            self.buffer.clear();
            return Ok(false);
        }
        self.buffer.resize(block_len as usize, 0u8);
        self.reader.read_exact(&mut self.buffer[..])?;
        Ok(true)
    }

    #[inline(always)]
    pub fn offset(&self) -> usize {
        self.offset
    }

    #[inline(always)]
    pub fn advance(&mut self, num_bytes: usize) {
        self.offset += num_bytes;
    }

    #[inline(always)]
    pub fn buffer(&self) -> &[u8] {
        &self.buffer[self.offset..]
    }
}

impl<'a> io::Read for BlockReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = self.buffer().read(buf)?;
        self.advance(len);
        Ok(len)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        let len = self.buffer.len();
        buf.extend_from_slice(self.buffer());
        self.advance(len);
        Ok(len)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.buffer().read_exact(buf)?;
        self.advance(buf.len());
        Ok(())
    }
}
