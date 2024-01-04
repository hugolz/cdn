use crate::{data::Metadata, error::CacheError};
use rocket::serde::json::serde_json::{self, json};
use rocket::tokio::{self, io::AsyncWriteExt as _};
use std::{
    str::FromStr,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    }, io::Read,
};

const CACHE_DIRECTORY: &'static str = "./cache";

#[derive(Debug)]
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
    // Thread pool, we'll use async executor
    pub data: Vec<Arc<CacheEntry>>,
    // cache_dir : std::path::Path,
    // Zip archive instead of path ?
}

impl Cache {
    pub fn new() -> Self {
        let files = std::fs::read_dir(CACHE_DIRECTORY).expect("Could not read cache directory");

        let data = files
            .flatten()
            .flat_map(|entry| {
                let metadata = entry.metadata().unwrap();
                if metadata.is_dir() {
                    return None;
                }
                let path = entry.path();
                let id = path.file_stem().unwrap().to_str().unwrap();
                let ext = path.extension().unwrap().to_str().unwrap();
                // ignore data files
                if ext == "data" {
                    return None;
                }

                // debug!("{id}, {ext}");
                let file_content: serde_json::Value =
                    serde_json::from_str(&std::fs::read_to_string(path.clone()).unwrap()).unwrap();
                let username = file_content
                    .get("username")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string();
                let file_ext = file_content
                    .get("extension")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string();
                let data_size = file_content
                    .get("data size")
                    .unwrap()
                    .as_number()
                    .unwrap()
                    .as_u64()
                    .unwrap() as usize;

                Some(Arc::new(CacheEntry {
                    id: uuid::Uuid::from_str(id).unwrap(),
                    metadata: Metadata { username, file_ext },
                    is_ready: AtomicBool::new(true),
                    data_size: AtomicUsize::new(data_size),
                }))
            })
            .collect::<Vec<Arc<CacheEntry>>>();
        debug!("{data:?}");

        Self { data }
    }
    pub fn store(
        &mut self,
        id: uuid::Uuid,
        meta: Metadata,
        data: Vec<u8>,
    ) -> tokio::task::JoinHandle<Result<(), CacheError>> {
        // Compress and store the given cache entry
        let entry = Arc::new(CacheEntry::new(id, meta));
        self.data.push(entry.clone());

        tokio::spawn(async move { store(entry, data).await })
    }

    pub async fn load(&mut self, id: uuid::Uuid) -> Result<(Metadata, Vec<u8>), CacheError> {
        // Load and decompress the given cache entry
        let entry = self.data.iter().find(|e| e.id == id).ok_or(CacheError::NotFound)?;

        let mut raw_compressed = Vec::new();

        let mut _read = std::fs::File::open(format!("{CACHE_DIRECTORY}/{id}.data")).unwrap().read_to_end(&mut raw_compressed);

        let mut raw = Vec::new();
        brotli::BrotliDecompress(&mut std::io::Cursor::new(raw_compressed), &mut raw).unwrap();

        Ok((
            entry.metadata.clone(),
            raw
        ))

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

    let (files, exec_time) = time::timeit_async(|| async {
        futures::join!(
            async {
                Ok(
                    match tokio::fs::File::create(format!("{CACHE_DIRECTORY}/{id}.data")).await {
                        Ok(f) => f,
                        Err(e) => {
                            error!("[{id}] Could not create data file: {e}");
                            return Err(CacheError::FileCreate("data".to_string()));
                        }
                    },
                )
            },
            async {
                Ok(
                    match tokio::fs::File::create(format!("{CACHE_DIRECTORY}/{id}.meta")).await {
                        Ok(file) => file,
                        Err(e) => {
                            error!("[{id}] Could not create meta file: {e}");
                            return Err(CacheError::FileCreate("meta".to_string()));
                        }
                    },
                )
            }
        )
    })
    .await;

    // TODO: Fix this
    let (mut data_file, mut meta_file) = match files {
        (Ok(data_file), Ok(meta_file)) => (data_file, meta_file),
        (Ok(_), Err(e)) => return Err(e),
        (Err(e), Ok(_)) => return Err(e),
        (Err(de), Err(me)) => return Err(CacheError::Test),
    };

    let (res, data_exec_time) = time::timeit_async(|| async {
        let data_length = data.len();

        // Compression algorithms seems rly uneffective with most files
        let encoder_params = brotli::enc::BrotliEncoderParams {
            quality: 5,
            ..Default::default()
        };

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

        let (res, file_write_exec_time) =
            time::timeit_async(|| async { data_file.write_all(&compressed_data).await }).await;

        if let Err(e) = res {
            error!("[{id}] Error while writing data file {e}");
            panic!()
        }

        debug!(
            "[{id}] Successfully wrote its data, took {}",
            time::format(file_write_exec_time)
        );
        Ok(bytes)
    })
    .await;

    debug!("Data was written in {}", time::format(data_exec_time));

    let bytes = match res {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("{e}");
            return Err(CacheError::FileWrite("data".to_string()));
        }
    };

    /* -------------------------------------------------------------------------------
                                Meta file

        Has all the usefull infos about the data file.
        I's written at the end so the download method wont find partial data
    ------------------------------------------------------------------------------- */

    let (res, meta_write_exec_time) = time::timeit_async(|| async {
        let data = serde_json::to_string_pretty(&json!({
            "username": entry.metadata.username,
            "extension": entry.metadata.file_ext,
            "data size": bytes
        }))
        .unwrap();

        if let Err(e) = meta_file.write_all(data.as_bytes()).await {
            error!("Failled to write meta file: {e}");
            return Err(CacheError::FileWrite("meta".to_string()));
        }
        Ok(())
    })
    .await;

    if let Err(e) = res {
        error!("[{id}] Could not write meta file due to: {e}");
        panic!()
    }

    debug!(
        "[{id}] Successfully wrote meta file, took: {}",
        time::format(meta_write_exec_time)
    );
    debug!(
        "[{id}] Cache creation ended successfully in {:?}",
        total_timer.elapsed()
    );
    entry.is_ready.store(true, Ordering::Relaxed);
    Ok(())
}
