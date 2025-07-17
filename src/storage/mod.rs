use anyhow::Result;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use sled::Db;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub file_id: Uuid,
    pub filename: String,
    pub tags: Vec<String>,
    pub owner_peer_id: String,
    pub checksum: String,
    pub size: u64,
}

pub struct Storage {
    db: Db,
}

impl Storage {
    pub fn new(path: &str) -> Result<Self> {
        let db = sled::open(path)?;
        Ok(Self { db })
    }
    pub fn insert_metadata(&self, meta: &FileMetadata) -> Result<()> {
        let key = meta.file_id.as_bytes();
        let value = bincode::serialize(meta)?;
        self.db.insert(key, value)?;
        Ok(())
    }
    pub fn get_metadata(&self, file_id: &Uuid) -> Result<Option<FileMetadata>> {
        if let Some(val) = self.db.get(file_id.as_bytes())? {
            let meta: FileMetadata = bincode::deserialize(&val)?;
            Ok(Some(meta))
        } else {
            Ok(None)
        }
    }
    pub fn list_metadata(&self) -> Result<Vec<FileMetadata>> {
        let mut out = Vec::new();
        for item in self.db.iter() {
            let (_k, v) = item?;
            let meta: FileMetadata = bincode::deserialize(&v)?;
            out.push(meta);
        }
        Ok(out)
    }
} 