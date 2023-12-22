use crate::{data::Metadata, error::CacheError};
use rocket::serde::json::serde_json::{self, json};
use rocket::tokio::{self, io::AsyncWriteExt as _};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};

pub struct CacheEntry {
    id: uuid::Uuid,
    metadata: Metadata,
    is_ready: AtomicBool,
    data_size: AtomicUsize,
}

impl CacheEntry {
    pub fn new(id: uuid::Uuid, metadata: Metadata) -> Self {
        Self {
            id,
            metadata,
            is_ready: AtomicBool::new(false),
            data_size: AtomicUsize::new(0),
        }
    }
    pub fn is_ready(&self) -> bool {
        self.is_ready.load(Ordering::Relaxed)
    }
    pub fn set_ready(&mut self, rdy: bool) {
        self.is_ready.store(rdy, Ordering::Relaxed)
    }
}

#[derive(Default)]
pub struct Cache {
    /*
        This should store some sort of Metadata list
        and keep track of the one that are not compressed yet /
        the one that are not ready yet.

    */
    // Thread pool
    data: Vec<Arc<CacheEntry>>,
    // cache_dir : std::path::Path,
    // Zip archive instead of path ?
}

impl Cache {
    pub fn store(
        &mut self,
        id: uuid::Uuid,
        meta: Metadata,
        data: Vec<u8>,
    ) -> tokio::task::JoinHandle<Result<(), CacheError>> {
        // Compress and store the given cache entry
        let entry = Arc::new(CacheEntry::new(id, meta));
        // self.data.push(entry.clone());

        tokio::spawn(async move { store(entry, data).await })
    }

    pub fn load(&mut self, id: uuid::Uuid) -> Result<(Metadata, String), CacheError> {
        // Load and decompress the given cache entry

        Err(CacheError::Test)
    }
}

// pub struct CacheHandle {
//     /*
//         This will be used as a lightweight controller that we can use to

//         to save and ecrypt, just send the data and the cache save

//     */
// }

async fn store(entry: Arc<CacheEntry>, data: Vec<u8>) -> Result<(), CacheError> {
    use std::io::Cursor;

    let id = entry.id.as_hyphenated().to_string();
    let total_timer = std::time::Instant::now();
    let Ok(mut file) = tokio::fs::File::create(format!("./cache/{id}.data")).await else {
        error!("[{id}] Could not create data file");
        return Err(CacheError::FileCreate("data".to_string()));
    };
    let data_length = data.len();

    // Compression algorithms seems rly uneffective with most files

    let mut encoder_params = brotli::enc::BrotliEncoderParams::default();
    encoder_params.quality = 5;

    let mut compressed_data = Vec::new();
    let mut data_reader = Cursor::new(data);
    let (res, compression_exec_time) = time::timeit_mut(|| {
        brotli::BrotliCompress(&mut data_reader, &mut compressed_data, &encoder_params)
    });
    let bytes = match res {
        Ok(bytes) => {
            entry.data_size.store(bytes, Ordering::Relaxed);
            bytes
        }
        Err(e) => {
            error!("[{id}] Failled to compress due to: {e}");
            return Err(CacheError::Compression);
        }
    };

    debug!(
        "[{id}] Finished compressing {} -> {}, took {:?}",
        rocket::data::ByteUnit::Byte(data_length as u64),
        rocket::data::ByteUnit::Byte(bytes as u64),
        compression_exec_time
    );

    let file_write_timer = std::time::Instant::now();
    file.write_all(&compressed_data).await;
    debug!(
        "[{id}] Successfully wrote its data, took {}",
        time::display_duration(file_write_timer.elapsed())
    );

    /* -------------------------------------------------------------------------------
                                Meta file

        Has all the usefull infos about the data file.
        I's written at the end so the download method wont find partial data
    ------------------------------------------------------------------------------- */

    let mut file = match rocket::tokio::fs::File::create(format!("./cache/{id}.meta")).await {
        Ok(file) => file,
        Err(e) => {
            error!("[{id}] Could not create meta file: {e}");
            return Err(CacheError::FileCreate("meta".to_string()));
        }
    };

    let meta_write_timer = std::time::Instant::now();
    if let Err(e) = file
        .write_all(
            serde_json::to_string_pretty(&json!({
                "username": entry.metadata.username,
                "extension": entry.metadata.file_ext,
                "data size": bytes
            }))
            .unwrap()
            .as_bytes(),
        )
        .await
    {
        error!("[{id}] Could not write meta file due to: {e}");
    }
    debug!(
        "[{id}] Successfully wrote meta file, took: {:?}",
        meta_write_timer.elapsed()
    );
    debug!(
        "[{id}] Cache creation ended successfully in {:?}",
        total_timer.elapsed()
    );
    entry.is_ready.store(true, Ordering::Relaxed);
    Ok(())
}
