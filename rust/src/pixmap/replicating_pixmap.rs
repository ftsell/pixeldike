use super::*;
use anyhow::{Context, Result};
use std::sync::{mpsc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const LOG_TARGET: &str = "pixelflut.pixmap.replica";

#[derive(Debug, Error)]
enum Error {
    #[error("could not verify that primary and replica sizes are compatible")]
    SizeVerificationError,
    #[error("a replication thread is already started")]
    ReplicationThreadAlreadyStarted,
}

type ReplicaList = Vec<Box<dyn Pixmap + Send + Sync>>;

struct ReplicationThreadHandler {
    #[allow(dead_code)]
    join_handle: Option<thread::JoinHandle<()>>,
    stop_channel: mpsc::Sender<()>,
}

///
/// A pixmap implementation which periodically replicates the state of one pixmap to one or
/// more replicas.
///
pub struct ReplicatingPixmap<P>
where
    P: Pixmap + Send + Sync + 'static,
{
    primary: Arc<P>,
    replicas: Arc<ReplicaList>,
    frequency: f64,
    replication_thread: Option<Mutex<ReplicationThreadHandler>>,
}

impl<P> ReplicatingPixmap<P>
where
    P: Pixmap + Send + Sync + 'static,
{
    pub fn new(primary: P, replicas: ReplicaList, frequency: f64) -> Result<Self> {
        match primary.get_size() {
            Ok(primary_size) => {
                // iterate over all replicas and verify that they have the same size as the primary
                for replica in &replicas {
                    match replica.get_size() {
                        Ok(replica_size) => {
                            if replica_size != primary_size {
                                return Err(Error::SizeVerificationError)
                                    .context(format!("replica size is not equal to primary size"));
                            }
                        }
                        Err(e) => {
                            return Err(Error::SizeVerificationError)
                                .context(format!("tried to retrieve size of a replica"))
                                .context(e)
                        }
                    }
                }

                let mut pixmap = Self {
                    primary: Arc::new(primary),
                    replicas: Arc::new(replicas),
                    frequency,
                    replication_thread: None,
                };
                pixmap.start_replication()?;

                Ok(pixmap)
            }
            Err(e) => Err(Error::SizeVerificationError)
                .context(format!("tried to retrieve size of primary"))
                .context(e),
        }
    }

    pub(self) fn replicate(primary: &Arc<P>, replicas: &Arc<ReplicaList>) -> Result<()> {
        let data = primary
            .get_raw_data()
            .context("retrieving data from pixmap for replication")?;

        for replica in replicas.iter() {
            replica
                .put_raw_data(&data)
                .context("replicating data into pixmap")?;
        }

        Ok(())
    }

    fn start_replication(&mut self) -> Result<()> {
        if self.replication_thread.is_some() {
            return Err(Error::ReplicationThreadAlreadyStarted.into());
        }

        let primary = self.primary.clone();
        let replicas = self.replicas.clone();
        let frequency = self.frequency;
        let (sender, receiver) = mpsc::channel();

        let join_handle = thread::Builder::new()
            .name("pixelflut.replicator".to_string())
            .spawn(move || {
                let interval_duration = Duration::from_secs_f64(1.0 / frequency);
                debug!(
                    target: LOG_TARGET,
                    "Starting pixmap replication once every {:?}", interval_duration
                );

                // loop while nothing has been sent over the notification channel
                while receiver.try_recv().is_err() {
                    let start_time = Instant::now();

                    if let Err(e) = ReplicatingPixmap::replicate(&primary, &replicas)
                        .context("replicating in replication thread")
                    {
                        error!("{}", e);
                        return;
                    }

                    if start_time.elapsed() < interval_duration {
                        thread::sleep(interval_duration - start_time.elapsed())
                    }
                }
            })
            .expect("could not start replicator thread");

        self.replication_thread = Some(Mutex::new(ReplicationThreadHandler {
            join_handle: Some(join_handle),
            stop_channel: sender,
        }));

        Ok(())
    }

    fn stop_replication(&mut self) {
        if let Some(mutex) = self.replication_thread.take() {
            let mut replication_thread = mutex.lock().unwrap();
            replication_thread
                .stop_channel
                .send(())
                .expect("could not send stop signal to replication thread");
            replication_thread
                .join_handle
                .take()
                .unwrap()
                .join()
                .expect("could not join with replication thread");
            self.replication_thread = None;
        }
    }
}

impl<P> Drop for ReplicatingPixmap<P>
where
    P: Pixmap + Send + Sync + 'static,
{
    fn drop(&mut self) {
        self.stop_replication()
    }
}

impl<P> Pixmap for ReplicatingPixmap<P>
where
    P: Pixmap + Send + Sync,
{
    fn get_pixel(&self, x: usize, y: usize) -> Result<Color> {
        self.primary.get_pixel(x, y)
    }

    fn set_pixel(&self, x: usize, y: usize, color: Color) -> Result<()> {
        self.primary.set_pixel(x, y, color)
    }

    fn get_size(&self) -> Result<(usize, usize)> {
        self.primary.get_size()
    }

    fn get_raw_data(&self) -> Result<Vec<Color>> {
        self.primary.get_raw_data()
    }

    fn put_raw_data(&self, data: &Vec<Color>) -> Result<()> {
        self.primary.put_raw_data(data)
    }
}

#[cfg(test)]
mod test {
    use super::super::test;
    use super::*;
    use quickcheck::TestResult;
    use std::thread::sleep;
    use std::time::Duration;

    quickcheck! {
        fn test_set_and_get_pixel(width: usize, height: usize, x: usize, y: usize, color: Color) -> TestResult {
            if let (Ok(primary), Ok(secondary)) = (InMemoryPixmap::new(width, height), InMemoryPixmap::new(width, height)) {
                if let Ok(pixmap) = ReplicatingPixmap::new(primary, vec![Box::new(secondary)], 1.0) {
                    test::test_set_and_get_pixel(pixmap, x, y, color)
                } else {
                    TestResult::discard()
                }
            } else {
                TestResult::discard()
            }
        }
    }

    quickcheck! {
        fn test_put_and_get_raw_data(color: Color) -> TestResult {
            let pixmap = ReplicatingPixmap::new(
                InMemoryPixmap::default(),
                vec![Box::new(InMemoryPixmap::default())],
                1.0
            ).unwrap();
            test::test_put_and_get_raw_data(&pixmap, color)
        }
    }

    #[test]
    fn test_replicate() {
        let pixmap = ReplicatingPixmap::new(
            InMemoryPixmap::default(),
            vec![Box::new(InMemoryPixmap::default())],
            1.0,
        )
        .unwrap();

        pixmap.set_pixel(42, 42, Color(42, 42, 42)).unwrap();
        ReplicatingPixmap::replicate(&pixmap.primary, &pixmap.replicas).unwrap();

        for replica in pixmap.replicas.iter() {
            assert_eq!(replica.get_pixel(42, 42).unwrap(), Color(42, 42, 42))
        }
    }

    #[test]
    fn test_background_replicate() {
        let pixmap = ReplicatingPixmap::new(
            InMemoryPixmap::default(),
            vec![Box::new(InMemoryPixmap::default())],
            2.0,
        )
        .unwrap();

        pixmap.set_pixel(42, 42, Color(42, 42, 42)).unwrap();
        sleep(Duration::from_millis(300));

        for replica in pixmap.replicas.iter() {
            assert_eq!(replica.get_pixel(42, 42).unwrap(), Color(42, 42, 42))
        }
    }
}
