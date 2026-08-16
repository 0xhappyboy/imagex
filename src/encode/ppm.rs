use crate::Imagex;
use std::fs::File;
use std::io::Write;
use std::path::Path;
impl Imagex {
    /// Write image as PPM file (P6 format)
    pub fn write_ppm<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let mut file = File::create(path)?;
        writeln!(file, "P6")?;
        writeln!(file, "{} {}", self.width, self.height)?;
        writeln!(file, "255")?;
        let raw_data = self.as_raw();
        let pixel_stride = (self.width as usize) * 4;
        let buffer_stride = self.stride as usize;
        for y in 0..self.height as usize {
            let row_start = y * buffer_stride;
            let row_end = row_start + pixel_stride;
            for chunk in raw_data[row_start..row_end].chunks_exact(4) {
                file.write_all(&[chunk[0], chunk[1], chunk[2]])?;
            }
        }
        Ok(())
    }
}
