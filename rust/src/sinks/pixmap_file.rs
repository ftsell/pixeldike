//! A sink for periodically snapshotting the canvas into a pixmap file

use crate::pixmap::{Pixmap, SharedPixmap};
use crate::DaemonHandle;
use anyhow::anyhow;
use itertools::Itertools;
use std::io::SeekFrom;
use std::mem;
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tokio::time::Interval;

const FILE_MAGIC: &[u8] = b"PIXELFLUT";
const HEADER_SIZE: usize = mem::size_of::<u64>() * 2; // enough space for width and height

const SEEK_MAGIC: SeekFrom = SeekFrom::Start(0);
const SEEK_HEADER: SeekFrom = SeekFrom::Start(FILE_MAGIC.len() as u64);
const SEEK_DATA: SeekFrom = SeekFrom::Start((FILE_MAGIC.len() + HEADER_SIZE) as u64);

/// Configuration options for the [`FileSink`]
#[derive(Debug)]
pub struct FileSinkOptions {
    /// The interval between save iterations
    ///
    /// Every time the interval triggers, a snapshot is taken
    pub interval: Interval,

    /// The path at which the snapshot should be placed
    pub path: PathBuf,
}

/// A sink that periodically snapshots pixmap data into a file
#[derive(Debug)]
pub struct FileSink {
    options: FileSinkOptions,
    pixmap: SharedPixmap,
}

impl FileSink {
    /// Create a new file sink which sinks data from the given pixmap into a file
    pub fn new(options: FileSinkOptions, pixmap: SharedPixmap) -> Self {
        Self { options, pixmap }
    }

    /// Open the target file and start the background tasks for periodic snapshotting
    pub async fn start(self) -> anyhow::Result<DaemonHandle> {
        let mut file = self.open_file().await?;
        self.write_header(&mut file).await?;
        let handle = tokio::spawn(async move { self.run(file).await });
        Ok(DaemonHandle::new(handle))
    }

    /// Open the configured file for writing
    async fn open_file(&self) -> anyhow::Result<File> {
        Ok(File::options()
            .write(true)
            .create(true)
            .open(&self.options.path)
            .await?)
    }

    /// Write appropriate header information into the file so that later operations only have to write data
    async fn write_header(&self, file: &mut File) -> anyhow::Result<()> {
        // set file length to exact content size
        let (width, height) = self.pixmap.get_size();
        file.set_len((FILE_MAGIC.len() + HEADER_SIZE + width * height * 3) as u64)
            .await?;

        // write magic bytes
        file.seek(SEEK_MAGIC).await?;
        file.write_all(FILE_MAGIC).await?;

        // write actual header
        file.seek(SEEK_HEADER).await?;
        file.write_u64(width as u64).await?;
        file.write_u64(height as u64).await?;

        // sync data to disk
        file.flush().await?;
        file.sync_all().await?;
        Ok(())
    }

    /// Write pixmap data into the data section of the file
    async fn write_data(&self, file: &mut File) -> anyhow::Result<()> {
        file.seek(SEEK_DATA).await?;

        let data = unsafe { self.pixmap.get_color_data() };
        let data = data
            .iter()
            .flat_map(|c| Into::<[u8; 3]>::into(*c))
            .collect::<Vec<_>>();
        file.write_all(&data).await?;

        file.flush().await?;
        file.sync_all().await?;

        Ok(())
    }

    /// Execute the main loop which periodically snapshots data into the file
    async fn run(mut self, mut file: File) -> anyhow::Result<!> {
        loop {
            self.write_data(&mut file).await?;
            self.options.interval.tick().await;
        }
    }
}

/// Restore a previously saved pixmap snapshot
pub async fn load_pixmap_file(path: &Path) -> anyhow::Result<Pixmap> {
    let mut file = File::open(path).await?;

    // verify magic bytes
    let mut file_magic = [0u8; FILE_MAGIC.len()];
    file.seek(SEEK_MAGIC).await?;
    file.read_exact(&mut file_magic).await?;
    if file_magic != FILE_MAGIC {
        return Err(anyhow!(
            "File at {} does not contain valid pixmap data",
            path.display()
        ));
    }

    // load size information from header
    file.seek(SEEK_HEADER).await?;
    let width = file.read_u64().await? as usize;
    let height = file.read_u64().await? as usize;

    // load the file data into memory
    let mut buf = vec![0u8; width * height * 3];
    file.seek(SEEK_DATA).await?;
    file.read_exact(&mut buf).await?;

    // construct a pixmap with the loaded data
    let pixmap = Pixmap::new(width, height)?;
    let pixmap_data = unsafe { pixmap.get_color_data() };
    for (i, i_color) in buf.into_iter().tuples::<(_, _, _)>().enumerate() {
        pixmap_data[i] = i_color.into()
    }

    Ok(pixmap)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::pixmap::Color;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::time::interval;

    #[tokio::test]
    async fn test_store_and_load() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.into_path().join("test.pixmap");
        let original_pixmap = Arc::new(Pixmap::new(5, 5).unwrap());
        original_pixmap.set_pixel(0, 0, Color(0xab, 0xab, 0xab)).unwrap();
        original_pixmap.set_pixel(2, 2, Color(0xab, 0xab, 0xab)).unwrap();
        original_pixmap.set_pixel(4, 4, Color(0xab, 0xab, 0xab)).unwrap();

        // write data into the file
        {
            let sink = FileSink::new(
                FileSinkOptions {
                    path: file_path.clone(),
                    interval: interval(Duration::from_secs(1)),
                },
                original_pixmap.clone(),
            );
            let mut file = sink.open_file().await.unwrap();
            sink.write_header(&mut file).await.unwrap();
            sink.write_data(&mut file).await.unwrap();
        }

        // restore data from the file
        let restored_pixmap = load_pixmap_file(&file_path).await.unwrap();

        // compare data
        let original_data = unsafe { original_pixmap.get_color_data() };
        let restored_data = unsafe { restored_pixmap.get_color_data() };
        assert_eq!(original_data, restored_data);
    }
}
