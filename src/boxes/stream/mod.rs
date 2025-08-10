//! StreamBox 🌊 - ストリーミング処理
// Nyashの箱システムによるストリーミング処理を提供します。
// 参考: 既存Boxの設計思想

use std::io::{Read, Write, Result};

pub struct StreamBox<R: Read, W: Write> {
    pub reader: R,
    pub writer: W,
}

impl<R: Read, W: Write> StreamBox<R, W> {
    pub fn new(reader: R, writer: W) -> Self {
        StreamBox { reader, writer }
    }
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.reader.read(buf)
    }
    pub fn write(&mut self, buf: &[u8]) -> Result<()> {
        self.writer.write_all(buf)
    }
}
