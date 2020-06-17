pub mod error;

mod bucket;
mod headers;

pub use self::{
    error::{RatelimitError, RatelimitResult},
    headers::RatelimitHeaders,
};

use self::bucket::{Bucket, BucketQueueTask};
use crate::routing::Path;
use futures_channel::oneshot::{self, Receiver, Sender};
use futures_util::lock::Mutex;
use log::debug;
use std::{
    collections::hash_map::{Entry, HashMap},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

// Global lock. We use a pair to avoid actually locking the mutex every check.
// This allows futures to only wait on the global lock when a global ratelimit
// is in place by, in turn, waiting for a guard, and then each immediately
// dropping it.
#[derive(Debug, Default)]
struct GlobalLockPair(Mutex<()>, AtomicBool);

impl GlobalLockPair {
    pub fn lock(&self) {
        self.1.store(true, Ordering::Release);
    }

    pub fn unlock(&self) {
        self.1.store(false, Ordering::Release);
    }

    pub fn is_locked(&self) -> bool {
        self.1.load(Ordering::Relaxed)
    }
}

#[derive(Debug, Default)]
pub struct Ratelimiter {
    buckets: Arc<Mutex<HashMap<Path, Arc<Bucket>>>>,
    global: Arc<GlobalLockPair>,
}

impl Ratelimiter {
    /// Create a new ratelimiter.
    ///
    /// Most users won't need to use this directly. If you're creating your own
    /// HTTP proxy then this is good to use for your own ratelimiting.
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn get(&self, path: Path) -> Receiver<Sender<Option<RatelimitHeaders>>> {
        debug!("Getting bucket for path: {:?}", path);

        let (tx, rx) = oneshot::channel();
        let (bucket, fresh) = self.entry(path.clone(), tx).await;

        if fresh {
            tokio::spawn(
                BucketQueueTask::new(
                    bucket,
                    Arc::clone(&self.buckets),
                    Arc::clone(&self.global),
                    path,
                )
                .run(),
            );
        }

        rx
    }

    async fn entry(
        &self,
        path: Path,
        tx: Sender<Sender<Option<RatelimitHeaders>>>,
    ) -> (Arc<Bucket>, bool) {
        // nb: not realisically point of contention
        let mut buckets = self.buckets.lock().await;

        match buckets.entry(path.clone()) {
            Entry::Occupied(bucket) => {
                debug!("Got existing bucket: {:?}", path);

                let bucket = bucket.into_mut();
                bucket.queue.push(tx);
                debug!("Added request into bucket queue: {:?}", path);

                (Arc::clone(&bucket), false)
            }
            Entry::Vacant(entry) => {
                debug!("Making new bucket for path: {:?}", path);
                let bucket = Bucket::new(path.clone());
                bucket.queue.push(tx);

                let bucket = Arc::new(bucket);
                entry.insert(Arc::clone(&bucket));

                (bucket, true)
            }
        }
    }
}
