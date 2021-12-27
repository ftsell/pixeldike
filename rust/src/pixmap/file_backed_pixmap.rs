use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use anyhow::Result;
use byteorder::{ReadBytesExt, WriteBytesExt};
use thiserror::Error;

use super::*;

// TODO Implement handling of read-only files
// TODO Think of an implementation with RwLock instead of Mutex

const LOG_TARGET: &str = "pixelflut.pixmap.file";
const HEADER_SPACE: usize = 256;
const MAGIC_BYTES: [u8; 10] = [
    'P' as u8, 'I' as u8, 'X' as u8, 'E' as u8, 'L' as u8, 'F' as u8, 'L' as u8, 'U' as u8, 'T' as u8, 1,
];

const SEEK_MAGIC: SeekFrom = SeekFrom::Start(0);
const SEEK_HEADER: SeekFrom = SeekFrom::Start(MAGIC_BYTES.len() as u64);
const SEEK_DATA: SeekFrom = SeekFrom::Start((MAGIC_BYTES.len() + HEADER_SPACE) as u64);

#[derive(Error, Debug)]
pub enum Error {
    #[error("content of existing file is not a valid pixmap file")]
    InvalidFileType,
    #[error("the existing file contains pixmap data of different size than the requested pixmap")]
    IncompatiblePixmapData,
}

#[derive(Debug, Eq, PartialEq)]
pub struct FileHeader {
    width: u64,
    height: u64,
}

///
/// Pixmap implementation which reads and writes all data directly into a backing file
///
#[derive(Debug)]
pub struct FileBackedPixmap {
    file: Mutex<File>,
    header: FileHeader,
}

impl FileBackedPixmap {
    /// Create a new instance backed by a file at `path` with the specified size.
    ///
    /// If a file already exists at the destination that **is not** an existing pixmap of the same
    /// size, this only succeed when `overwrite` is true which then overwrites the existing file and thus
    /// removing all preexisting data from it.
    ///
    /// If a file already exists at the destination that **is** an existing pixmap of the same size
    /// and `overwrite` is true, the content of that file will be overwritten too and thus
    /// removing all preexisting pixel data from it.
    pub fn new(path: &Path, width: usize, height: usize, overwrite: bool) -> Result<Self> {
        if width == 0 || height == 0 {
            return Err(GenericError::InvalidSize(width, height).into());
        }

        // create containing directory hierarchy if it does not yet exist
        match path.parent() {
            Some(parent_dir) => create_dir_all(parent_dir)?,
            None => {}
        }
        let is_preexisting = path.exists();

        // create the resulting instance
        let instance = Self {
            file: Mutex::new(
                OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(path)?,
            ),
            header: FileHeader {
                width: width as u64,
                height: height as u64,
            },
        };

        {
            let mut lock = instance.file.lock().unwrap();
            let mut initial_data = vec![0; width * height * 3];

            // retrieve potential existing data for initialization or abort if necessary
            if is_preexisting {
                match instance.validate_magic_bytes(&mut lock) {
                    Ok(_) => {
                        // the file already is a pixmap file
                        let existing_header = instance.read_header(&mut lock)?;
                        if instance.header != existing_header {
                            // but it is incompatible
                            if !overwrite {
                                return Err(Error::IncompatiblePixmapData.into());
                            } else {
                                debug!(target: LOG_TARGET, "Overwriting data in existing file {:?}", path);
                            }
                        } else {
                            // and it is compatible
                            if !overwrite {
                                debug!(
                                    target: LOG_TARGET,
                                    "Reusing data from existing pixmap file {:?}", path
                                );
                                initial_data = instance.read_data(&mut lock)?;
                            } else {
                                debug!(
                                    target: LOG_TARGET,
                                    "Ignoring existing pixmap data from file {:?}", path
                                )
                            }
                        }
                    }
                    Err(e) => {
                        match e.downcast::<Error>() {
                            Ok(e) => match e {
                                Error::InvalidFileType => {
                                    // the file is accessible but not a pixmap file
                                    if !overwrite {
                                        return Err(Error::InvalidFileType.into());
                                    } else {
                                        debug!(
                                            target: LOG_TARGET,
                                            "Overwriting existing file {:?} with pixmap data", path
                                        )
                                    }
                                }
                                _ => return Err(e.into()), // some other of our errors
                            },
                            Err(e) => return Err(e), // some random other error
                        }
                    }
                }
            } else {
                debug!(target: LOG_TARGET, "Creating new file {:?} for pixmap data", path)
            }

            // write initial data into file
            lock.set_len((MAGIC_BYTES.len() + HEADER_SPACE + width * height * 3) as u64)?;
            instance.write_magic_bytes(&mut lock)?;
            instance.write_header(&mut lock)?;
            instance.write_data(&mut lock, &initial_data)?;
        }

        info!(target: LOG_TARGET, "Created file backed pixmap at {:?}", path);
        Ok(instance)
    }

    /// Validate that the file does contain pixelflut data by validating the magic bytes
    fn validate_magic_bytes(&self, lock: &mut MutexGuard<File>) -> Result<()> {
        if lock.metadata()?.len() < MAGIC_BYTES.len() as u64 {
            Err(Error::InvalidFileType.into())
        } else {
            let mut data = vec![0; MAGIC_BYTES.len()];
            lock.seek(SEEK_MAGIC)?;
            lock.read_exact(&mut data)?;

            if &data != &MAGIC_BYTES {
                Err(Error::InvalidFileType.into())
            } else {
                Ok(())
            }
        }
    }

    /// Write MAGIC_BYTES into the first bytes of the file
    fn write_magic_bytes(&self, lock: &mut MutexGuard<File>) -> Result<()> {
        lock.seek(SEEK_MAGIC)?;
        lock.write_all(&MAGIC_BYTES)?;
        Ok(())
    }

    /// Read and deserialize only the header part of the .pixmap file
    fn read_header(&self, lock: &mut MutexGuard<File>) -> Result<FileHeader> {
        lock.seek(SEEK_HEADER)?;
        Ok(FileHeader {
            width: (*lock).read_u64::<byteorder::BigEndian>()?,
            height: (*lock).read_u64::<byteorder::BigEndian>()?,
        })
    }

    /// Serialize and write only the header part of the .pixmap file
    fn write_header(&self, lock: &mut MutexGuard<File>) -> Result<()> {
        lock.seek(SEEK_HEADER)?;
        let mut buffer = BufWriter::new(&**lock);
        buffer.write_u64::<byteorder::BigEndian>(self.header.width)?;
        buffer.write_u64::<byteorder::BigEndian>(self.header.height)?;
        buffer.write_all(vec![0; HEADER_SPACE - 2 * 8].as_slice())?;
        buffer.flush()?;

        Ok(())
    }

    /// Read the complete data section from file
    fn read_data(&self, lock: &mut MutexGuard<File>) -> Result<Vec<u8>> {
        lock.seek(SEEK_DATA)?;
        let mut result = vec![0; (self.header.width * self.header.height * 3) as usize];
        lock.read_exact(&mut result)?;

        Ok(result)
    }

    /// Write the complete data section of the file
    fn write_data(&self, lock: &mut MutexGuard<File>, data: &[u8]) -> Result<()> {
        lock.seek(SEEK_DATA)?;
        lock.write_all(data)?;

        Ok(())
    }

    /// Read the data of a single pixel with from file
    fn read_pixel(&self, lock: &mut MutexGuard<File>, x: usize, y: usize) -> Result<[u8; 3]> {
        let seek_pixel = SeekFrom::Current((get_pixel_index(self, x, y)? * 3) as i64);

        lock.seek(SEEK_DATA)?;
        lock.seek(seek_pixel)?;
        let mut result = [0, 0, 0];
        lock.read_exact(&mut result)?;

        Ok(result)
    }

    /// Write a single pixel at into file.
    fn write_pixel(&self, lock: &mut MutexGuard<File>, x: usize, y: usize, color: [u8; 3]) -> Result<()> {
        let seek_pixel = SeekFrom::Current((get_pixel_index(self, x, y)? * 3) as i64);

        lock.seek(SEEK_DATA)?;
        lock.seek(seek_pixel)?;
        lock.write_all(&color)?;

        Ok(())
    }
}

impl Pixmap for FileBackedPixmap {
    fn get_pixel(&self, x: usize, y: usize) -> Result<Color> {
        if !are_coordinates_inside(self, x, y)? {
            Err(GenericError::InvalidCoordinates {
                target: (x, y),
                size: (self.header.width as usize, self.header.height as usize),
            }
            .into())
        } else {
            let mut lock = self.file.lock().unwrap();
            let bin_data = self.read_pixel(&mut lock, x, y).unwrap();
            Ok(Color(bin_data[0], bin_data[1], bin_data[2]))
        }
    }

    fn set_pixel(&self, x: usize, y: usize, color: Color) -> Result<()> {
        if !are_coordinates_inside(self, x, y)? {
            Err(GenericError::InvalidCoordinates {
                target: (x, y),
                size: (self.header.width as usize, self.header.height as usize),
            }
            .into())
        } else {
            let mut lock = self.file.lock().unwrap();
            Ok(self.write_pixel(&mut lock, x, y, [color.0, color.1, color.2])?)
        }
    }

    fn get_size(&self) -> Result<(usize, usize)> {
        Ok((self.header.width as usize, self.header.height as usize))
    }

    fn get_raw_data(&self) -> Result<Vec<Color>> {
        let mut result = Vec::new();

        let mut color = Color(0, 0, 0);
        let mut lock = self.file.lock().unwrap();
        for (i, byte) in self.read_data(&mut lock).unwrap().iter().enumerate() {
            if i % 3 == 0 {
                color.0 = byte.to_owned();
            } else if i % 3 == 1 {
                color.1 = byte.to_owned()
            } else if i % 3 == 2 {
                color.2 = byte.to_owned();
                result.push(color);
            }
        }

        Ok(result)
    }

    fn put_raw_data(&self, data: &Vec<Color>) -> Result<()> {
        let bin_data: Vec<u8> = data
            .iter()
            .flat_map(|color| vec![color.0, color.1, color.2])
            .collect();
        let mut lock = self.file.lock().unwrap();
        Ok(self.write_data(&mut lock, &bin_data).unwrap())
    }
}

#[cfg(test)]
mod test {
    use quickcheck::TestResult;
    use tempfile::tempdir;

    use super::super::test;
    use super::*;

    #[test]
    fn test_new_file() {
        // setup
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.pixmap");

        // execution
        let pixmap = FileBackedPixmap::new(&path, 800, 600, false);

        // verification
        assert!(pixmap.is_ok(), "pixmap creation failed: {:?}", pixmap);
        assert!(path.exists());
        assert_eq!(
            path.metadata().unwrap().len() as usize,
            MAGIC_BYTES.len() + HEADER_SPACE + 800 * 600 * 3
        );
    }

    #[test]
    fn test_overwriting_existing_incompatible_file() {
        // setup
        let dir = tempdir().unwrap();
        let smaller_path = dir.path().join("smaller.pixmap");
        let larger_path = dir.path().join("larger.pixmap");
        let different_path = dir.path().join("different.txt");
        let empty_path = dir.path().join("empty");
        {
            let _smaller_pixmap = FileBackedPixmap::new(&smaller_path, 100, 200, false).unwrap();
            let _larger_pixmap = FileBackedPixmap::new(&larger_path, 1000, 2000, false).unwrap();
            let mut different_file = OpenOptions::new()
                .create(true)
                .write(true)
                .open(&different_path)
                .unwrap();
            different_file
                .write_all("This is a text file".as_bytes())
                .unwrap();
            let mut _empty_file = OpenOptions::new()
                .create(true)
                .write(true)
                .open(&empty_path)
                .unwrap();
        }

        // execution + verification
        //for path in [smaller_path, larger_path, different_path, empty_path].iter() {
        for path in [empty_path].iter() {
            // execution (expected failure)
            let pixmap = FileBackedPixmap::new(path, 800, 600, false);
            // verification
            assert!(
                pixmap.is_err(),
                "pixmap creation did not fail although existing file is incompatible to new pixmap"
            );

            // execution (expected success)
            let pixmap = FileBackedPixmap::new(path, 800, 600, true);
            // verification
            assert!(pixmap.is_ok(), "pixmap creation failed: {:?}", pixmap);
            assert_eq!(
                path.metadata().unwrap().len() as usize,
                MAGIC_BYTES.len() + HEADER_SPACE + 800 * 600 * 3
            );
        }
    }

    #[test]
    fn test_reusing_existing_compatible_file() {
        // setup
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.pixmap");
        {
            let pixmap = FileBackedPixmap::new(&path, 800, 600, false).unwrap();
            pixmap.set_pixel(42, 42, Color(42, 42, 42)).unwrap();
        }

        // execution (without reset)
        let pixmap = FileBackedPixmap::new(&path, 800, 600, false);
        // verification
        assert!(pixmap.is_ok(), "pixmap creation failed: {:?}", pixmap);
        assert_eq!(pixmap.unwrap().get_pixel(42, 42).unwrap(), Color(42, 42, 42));

        // execution (with reset)
        let pixmap = FileBackedPixmap::new(&path, 800, 600, true);
        // verification
        assert!(pixmap.is_ok(), "pixmap creation failed: {:?}", pixmap);
        assert_eq!(pixmap.unwrap().get_pixel(42, 42).unwrap(), Color(0, 0, 0))
    }

    #[test]
    fn test_get_size() {
        // setup
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.pixmap");
        let pixmap = FileBackedPixmap::new(&path, 800, 600, false).unwrap();

        // execution
        let size = pixmap.get_size().unwrap();

        // verification
        assert_eq!(size, (800, 600));
    }

    #[test]
    fn test_fails_cleanly_on_io_error() {
        // setup
        let path = Path::new("/root/test.pixmap");

        // execution
        let pixmap = FileBackedPixmap::new(&path, 800, 600, false);

        // verification
        assert!(pixmap.is_err())
    }

    quickcheck! {
        fn test_set_and_get_pixel(x: usize, y: usize, color: Color) -> TestResult {
            let dir = tempdir().unwrap();
            let path = dir.path().join("test.pixmap");
            let pixmap = FileBackedPixmap::new(&path, 800, 600, true).unwrap();
            test::test_set_and_get_pixel(pixmap, x, y, color)
        }
    }

    #[test]
    fn test_put_and_get_raw_data() {
        // setup
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.pixmap");
        let pixmap = FileBackedPixmap::new(&path, 800, 600, true).unwrap();

        for i in vec![0, 1, 256, 257, 4096, 4097] {
            // execution
            let result = test::test_put_and_get_raw_data(&pixmap, i.into());

            // verification
            assert!(!result.is_error() && !result.is_failure())
        }
    }
}
