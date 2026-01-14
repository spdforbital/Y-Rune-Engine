use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

/// Request to load an asset in the background.
#[derive(Debug)]
pub enum LoadRequest {
    /// Load a texture from disk
    Texture { path: String },
    /// Shutdown the loader
    Shutdown,
}

/// Result of a background load operation.
#[derive(Debug)]
pub enum LoadResult {
    /// Texture data loaded from disk (raw bytes ready for GPU upload)
    TextureData {
        path: String,
        data: Vec<u8>,
        width: u32,
        height: u32,
    },
    /// Error loading an asset
    Error {
        path: String,
        message: String,
    },
}

/// Background asset loader using a worker thread.
/// Handles disk I/O asynchronously, returning raw data for GPU upload on main thread.
pub struct AssetLoader {
    sender: Sender<LoadRequest>,
    receiver: Receiver<LoadResult>,
    handle: Option<JoinHandle<()>>,
}

impl AssetLoader {
    /// Create a new AssetLoader with a background worker thread.
    pub fn new() -> Self {
        let (request_tx, request_rx) = mpsc::channel::<LoadRequest>();
        let (result_tx, result_rx) = mpsc::channel::<LoadResult>();

        let handle = thread::spawn(move || {
            Self::worker_loop(request_rx, result_tx);
        });

        Self {
            sender: request_tx,
            receiver: result_rx,
            handle: Some(handle),
        }
    }

    /// Worker loop that processes load requests.
    fn worker_loop(receiver: Receiver<LoadRequest>, sender: Sender<LoadResult>) {
        loop {
            match receiver.recv() {
                Ok(LoadRequest::Texture { path }) => {
                    let result = Self::load_texture_data(&path);
                    let _ = sender.send(result);
                }
                Ok(LoadRequest::Shutdown) | Err(_) => {
                    break;
                }
            }
        }
    }

    /// Load texture data from disk (CPU-side work).
    fn load_texture_data(path: &str) -> LoadResult {
        match image::open(path) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                let (width, height) = rgba.dimensions();
                LoadResult::TextureData {
                    path: path.to_string(),
                    data: rgba.into_raw(),
                    width,
                    height,
                }
            }
            Err(e) => LoadResult::Error {
                path: path.to_string(),
                message: e.to_string(),
            },
        }
    }

    /// Queue a texture for background loading.
    pub fn load_texture(&self, path: &str) {
        let _ = self.sender.send(LoadRequest::Texture {
            path: path.to_string(),
        });
    }

    /// Try to receive a completed load result (non-blocking).
    pub fn try_recv(&self) -> Option<LoadResult> {
        self.receiver.try_recv().ok()
    }

    /// Receive a completed load result (blocking).
    pub fn recv(&self) -> Option<LoadResult> {
        self.receiver.recv().ok()
    }

    /// Check if there are any pending results.
    pub fn has_pending(&self) -> bool {
        // Try to peek without consuming
        // Unfortunately mpsc doesn't have peek, so we just check if recv would block
        // For now, caller should use try_recv in a loop
        false
    }

    /// Shutdown the worker thread.
    pub fn shutdown(&mut self) {
        let _ = self.sender.send(LoadRequest::Shutdown);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for AssetLoader {
    fn drop(&mut self) {
        self.shutdown();
    }
}

impl Default for AssetLoader {
    fn default() -> Self {
        Self::new()
    }
}
