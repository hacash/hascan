use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TrySendError};

pub(crate) struct ScanInner {
    pub(crate) dbconn: Mutex<Connection>,
    pub(crate) setting: Mutex<ScanSettings>,
    pub(crate) last_error: Mutex<Option<String>>,
    /// Addresses whose ranking balance changed but is not persisted yet. The
    /// engine cannot serve a state snapshot while it moves its durable root, so
    /// a failed refresh keeps the work queued instead of dropping it.
    pending_ranking: Mutex<std::collections::HashSet<Address>>,
    view: Mutex<Option<Arc<dyn ScanerView>>>,
    catchup: Mutex<()>,
    wake_rx: Mutex<Option<Receiver<()>>>,
}

#[derive(Clone)]
pub struct BlkScaner {
    pub cnf: BlkScrConfig,
    pub(crate) inner: Arc<ScanInner>,
    wake_tx: SyncSender<()>,
}

impl BlkScaner {
    pub fn new(cnf: BlkScrConfig, setting: ScanSettings, dbconn: Connection) -> Ret<Self> {
        dbconn
            .execute_batch(
                "PRAGMA journal_mode=WAL;
                 PRAGMA temp_store=MEMORY;
                 PRAGMA cache_size=-65536;
                 PRAGMA wal_autocheckpoint=1000;",
            )
            .map_err(|e| sys::Error::fault(format!("configure hascan SQLite: {e}")))?;
        dbconn
            .pragma_update(None, "synchronous", &cnf.synchronous)
            .map_err(|e| sys::Error::fault(format!("set hascan synchronous mode: {e}")))?;
        dbconn
            .busy_timeout(Duration::from_secs(5))
            .map_err(|e| sys::Error::fault(format!("set hascan busy timeout: {e}")))?;
        let (wake_tx, wake_rx) = sync_channel(1);
        Ok(Self {
            cnf,
            inner: Arc::new(ScanInner {
                dbconn: Mutex::new(dbconn),
                setting: Mutex::new(setting),
                last_error: Mutex::new(None),
                pending_ranking: Mutex::new(std::collections::HashSet::new()),
                view: Mutex::new(None),
                catchup: Mutex::new(()),
                wake_rx: Mutex::new(Some(wake_rx)),
            }),
            wake_tx,
        })
    }

    fn remember_view(&self, view: Arc<dyn ScanerView>) {
        *self.inner.view.lock().unwrap() = Some(view);
    }

    fn set_error(&self, error: Option<String>) {
        *self.inner.last_error.lock().unwrap() = error;
    }
}

impl Scaner for BlkScaner {
    fn name(&self) -> &str {
        "hascan"
    }

    fn sync(&self, view: Arc<dyn ScanerView>) -> Rerr {
        self.remember_view(view.clone());
        match self.catch_up(view, true) {
            Ok(()) => {
                self.set_error(None);
                Ok(())
            }
            Err(error) => {
                self.set_error(Some(error.to_string()));
                Err(error)
            }
        }
    }

    fn on_block(&self, _block: BlockRef, view: Arc<dyn ScanerView>) -> Rerr {
        self.remember_view(view);
        match self.wake_tx.try_send(()) {
            Ok(()) | Err(TrySendError::Full(())) => {}
            Err(TrySendError::Disconnected(())) => {
                self.set_error(Some("hascan worker channel disconnected".to_owned()));
            }
        }
        Ok(())
    }

    fn api_services(&self) -> Vec<Arc<dyn ApiService>> {
        vec![crate::server::service(self.inner.clone())]
    }

    fn start(&self, waiter: Waiter) -> Rerr {
        let receiver = self
            .inner
            .wake_rx
            .lock()
            .unwrap()
            .take()
            .ok_or_else(|| sys::Error::fault("hascan worker already started"))?;
        let scaner = self.clone();
        std::thread::Builder::new()
            .name("hascan-indexer".to_owned())
            .spawn(move || {
                let Some(_hold) = waiter.try_hold() else {
                    return;
                };
                loop {
                    if waiter.is_shutdown() {
                        break;
                    }
                    match receiver.recv_timeout(Duration::from_millis(250)) {
                        Ok(()) => {}
                        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                            // A ranking refresh deferred by a busy state snapshot
                            // must not wait for the next stable block: catch-up is
                            // usually the only caller and can idle for a long time.
                            if scaner.inner.pending_ranking.lock().unwrap().is_empty() {
                                continue;
                            }
                        }
                        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                    }
                    let view = scaner.inner.view.lock().unwrap().clone();
                    let Some(view) = view else {
                        continue;
                    };
                    match scaner.catch_up(view, false) {
                        Ok(()) => scaner.set_error(None),
                        Err(error) => {
                            eprintln!("[hascan] catch-up failed: {error}");
                            scaner.set_error(Some(error.to_string()));
                        }
                    }
                }
            })
            .map_err(|e| sys::Error::fault(format!("spawn hascan worker failed: {e}")))?;
        Ok(())
    }
}
