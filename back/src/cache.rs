use crate::error::CacheError;
use rocket::serde::json::serde_json::{self, json};
use rocket::tokio::io::AsyncReadExt;
use rocket::tokio::{self, io::AsyncWriteExt as _};
use shared::data::{CacheEntry, Metadata};
use std::{
    str::FromStr,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
};

const CACHE_DIRECTORY: &'static str = "./cache";
const COMPRESSION_LEVEL: i32 = 5; // 1..=11 The only difference with 11 is that 11 takes more time

#[derive(Default)]
pub struct Cache {
    pub data: Vec<Arc<CacheEntry>>,
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

        tokio::spawn(async move {
            let original_data_length = data.len();
            let (res, exec_time) = time::timeit_async(|| async { store(entry, data).await }).await;

            res.map(|compressed_data_length| {
                debug!(
                    "[{id}] Cache was successfully compresed ({} -> {}) in {}",
                    rocket::data::ByteUnit::Byte(original_data_length as u64),
                    rocket::data::ByteUnit::Byte(compressed_data_length as u64),
                    time::format(exec_time)
                );
            })
        })
    }

    pub async fn load(&self, id: uuid::Uuid) -> Result<(Metadata, Vec<u8>), CacheError> {
        // Load and decompress the given cache entry
        let entry = self
            .data
            .iter()
            .find(|e| e.id == id)
            .ok_or(CacheError::NotFound)?;

        let mut raw_compressed = Vec::new();

        let file_path = format!("{CACHE_DIRECTORY}/{id}.data");

        if let Err(e) = tokio::fs::File::open(file_path.clone())
            .await
            .map_err(|_| CacheError::FileOpen(file_path))?
            .read_to_end(&mut raw_compressed)
            .await
        {
            error!("{e}");
            return Err(CacheError::FileRead(format!("{e}")));
        }

        let mut raw = Vec::new();
        if let Err(e) =
            brotli::BrotliDecompress(&mut std::io::Cursor::new(raw_compressed), &mut raw)
        {
            error!("[{id}]Decompression failed:{e}");
            return Err(CacheError::Decompression);
        }

        Ok((entry.metadata.clone(), raw))
    }
}

async fn store(entry: Arc<CacheEntry>, original_data: Vec<u8>) -> Result<usize, CacheError> {
    use std::io::Cursor;

    let id = entry.id.as_hyphenated().to_string();

    let files = futures::join!(
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
    );

    // TODO: Fix this
    let (mut data_file, mut meta_file) = match files {
        (Ok(data_file), Ok(meta_file)) => (data_file, meta_file),
        (Ok(_), Err(e)) => return Err(e),
        (Err(e), Ok(_)) => return Err(e),
        (Err(de), Err(me)) => {
            return Err(CacheError::FileCreate(format!("Data ({de})\nMeta ({me})")))
        }
    };

    /* -------------------------------------------------------------------------------
                                Data file

        Stores a compressed version of the given data.
    ------------------------------------------------------------------------------- */

    // Compression algorithms seems rly uneffective with most files
    let encoder_params = brotli::enc::BrotliEncoderParams {
        quality: COMPRESSION_LEVEL,
        ..Default::default()
    };

    let mut compressed_data_buffer = Vec::new();
    let mut original_data_reader = Cursor::new(original_data);
    let compression_result = brotli::BrotliCompress(
        &mut original_data_reader,
        &mut compressed_data_buffer,
        &encoder_params,
    );

    let compressed_data_length = match compression_result {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("[{id}] Failled to compress data due to: {e}");
            return Err(CacheError::Compression);
        }
    };

    // Don't forget to update the cache entry
    entry
        .data_size
        .store(compressed_data_length, Ordering::Relaxed);

    let data_write_result = data_file.write_all(&compressed_data_buffer).await;

    if let Err(e) = data_write_result {
        error!("[{id}] Error while writing data file {e}");
        return Err(CacheError::FileWrite(format!("data - {e}")));
    }

    /* -------------------------------------------------------------------------------
                                Meta file

        Has all the usefull infos about the data file.
        It's written at the end so the download method wont find partial data
        -- partially true, we mainly have a 'ready' atomic bool for this
    ------------------------------------------------------------------------------- */

    let meta = serde_json::to_string_pretty(&json!({
        "username": entry.metadata.username,
        "extension": entry.metadata.file_ext,
        "data size": compressed_data_length
    }))
    .unwrap();

    let meta = meta.as_bytes(); // Cannot inline with the above due to lifetime issues

    if let Err(e) = meta_file.write_all(meta).await {
        error!("Failled to write meta file: {e}");
        return Err(CacheError::FileWrite(format!("meta - {e}")));
    }

    entry.is_ready.store(true, Ordering::Relaxed);

    Ok(compressed_data_length)
}
