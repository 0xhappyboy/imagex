use crate::Imagex;
use std::fs::File;
use std::io::Write;
use std::path::Path;
impl Imagex {
    /// Write image as PNG file
    pub fn write_png<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        use miniz_oxide::deflate::compress_to_vec_zlib;
        let mut file = File::create(path)?;
        // PNG signature
        file.write_all(b"\x89PNG\r\n\x1A\n")?;
        // IHDR chunk
        let mut ihdr = [0u8; 13];
        ihdr[0..4].copy_from_slice(&self.width.to_be_bytes());
        ihdr[4..8].copy_from_slice(&self.height.to_be_bytes());
        ihdr[8] = 8; // bit depth
        ihdr[9] = 6; // color type: RGBA
        ihdr[10] = 0; // compression method
        ihdr[11] = 0; // filter method
        ihdr[12] = 0; // interlace method
        // Write IHDR chunk
        let len = (13u32).to_be_bytes();
        file.write_all(&len)?;
        file.write_all(b"IHDR")?;
        file.write_all(&ihdr)?;
        let mut crc_data = Vec::with_capacity(4 + 13);
        crc_data.extend_from_slice(b"IHDR");
        crc_data.extend_from_slice(&ihdr);
        let crc = super::crc32(&crc_data);
        file.write_all(&crc.to_be_bytes())?;
        // Prepare raw RGBA data - copy only valid pixels, skip stride padding
        let raw_data = self.as_raw();
        let pixel_stride = (self.width as usize) * 4;
        let buffer_stride = self.stride as usize;
        let height = self.height as usize;
        // Build filtered data with filter bytes
        let mut filtered_data = Vec::with_capacity(height * (pixel_stride + 1));
        for y in 0..height {
            // Filter byte: 0 = None (no filtering)
            filtered_data.push(0);
            // Calculate start and end positions in the buffer
            let row_start = y * buffer_stride;
            let row_end = row_start + pixel_stride;
            // Copy only the valid pixel data (skip stride padding)
            filtered_data.extend_from_slice(&raw_data[row_start..row_end]);
        }
        // Compress the filtered data with zlib (use level 6 default)
        let compressed = compress_to_vec_zlib(&filtered_data, 6);
        // Write IDAT chunk
        let len = (compressed.len() as u32).to_be_bytes();
        file.write_all(&len)?;
        file.write_all(b"IDAT")?;
        file.write_all(&compressed)?;
        // Calculate CRC for IDAT chunk (type + data)
        let mut crc_data = Vec::with_capacity(4 + compressed.len());
        crc_data.extend_from_slice(b"IDAT");
        crc_data.extend_from_slice(&compressed);
        let crc = super::crc32(&crc_data);
        file.write_all(&crc.to_be_bytes())?;
        // IEND chunk
        file.write_all(&0u32.to_be_bytes())?;
        file.write_all(b"IEND")?;
        file.write_all(&0xAE426082u32.to_be_bytes())?;
        Ok(())
    }
}
