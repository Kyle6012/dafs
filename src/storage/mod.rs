use anyhow::Result;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use sled::Db;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub file_id: Uuid,
    pub filename: String,
    pub tags: Vec<String>,
    pub owner_peer_id: String,
    pub checksum: String,
    pub size: u64,
    pub encrypted_file_key: Vec<u8>, // new field
    pub shared_keys: HashMap<String, Vec<u8>>, // username -> encrypted file key
    pub allowed_peers: Vec<String>, // peer IDs allowed to access this file
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
    
    pub fn delete_metadata(&self, file_id: &Uuid) -> Result<()> {
        self.db.remove(file_id.as_bytes())?;
        Ok(())
    }
} 

impl Default for Storage {
    fn default() -> Self {
        Storage::new("dafs_db").expect("Failed to open default dafs_db")
    }
} 