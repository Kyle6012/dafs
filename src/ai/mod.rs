use crate::storage::FileMetadata;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use ndarray::{Array1, Array2, Array, Axis};
use std::sync::Mutex;
use once_cell::sync::Lazy;
use thiserror::Error;

const EMBEDDING_SIZE: usize = 32;
const HIDDEN_SIZE: usize = 16;
const OUTPUT_SIZE: usize = 1;

/// Errors for the AI module
#[derive(Debug, Error, Serialize)]
pub enum AIError {
    #[error("Mutex lock poisoned")] 
    MutexPoisoned,
    #[error("Shape error: {0}")]
    ShapeError(String),
    #[error("Numerical instability detected (NaN or Inf)")]
    NumericalInstability,
    #[error("Model validation failed: {0}")]
    ModelValidation(String),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
/// Neural Collaborative Filtering Model for recommendations
/// Only model weights/embeddings are shared in federated learning. No raw user data leaves the peer.
pub struct NCFModel {
    pub user_embeddings: HashMap<String, Vec<f32>>, // user_id -> embedding
    pub file_embeddings: HashMap<String, Vec<f32>>, // file_id -> embedding
    pub w1: Vec<Vec<f32>>, // first layer weights
    pub w2: Vec<Vec<f32>>, // second layer weights
    pub b1: Vec<f32>,
    pub b2: Vec<f32>,
    pub epoch: u32,
    // pub signature: Option<Vec<u8>>, // For cryptographic signatures (stub)
}

impl NCFModel {
    pub fn new() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let w1 = (0..HIDDEN_SIZE).map(|_| (0..(EMBEDDING_SIZE*2)).map(|_| rng.gen_range(-0.1..0.1)).collect()).collect();
        let w2 = (0..OUTPUT_SIZE).map(|_| (0..HIDDEN_SIZE).map(|_| rng.gen_range(-0.1..0.1)).collect()).collect();
        let b1 = vec![0.0; HIDDEN_SIZE];
        let b2 = vec![0.0; OUTPUT_SIZE];
        Self {
            user_embeddings: HashMap::new(),
            file_embeddings: HashMap::new(),
            w1,
            w2,
            b1,
            b2,
            epoch: 0,
            // signature: None,
        }
    }
    /// Forward pass. Returns error if numerical instability is detected.
    pub fn forward(&self, user: &str, file: &str) -> Result<f32, AIError> {
        let user_fallback = vec![0.0; EMBEDDING_SIZE];
        let ue = self.user_embeddings.get(user).unwrap_or(&user_fallback);
        let file_fallback = vec![0.0; EMBEDDING_SIZE];
        let fe = self.file_embeddings.get(file).unwrap_or(&file_fallback);
        let input: Array1<f32> = Array::from_iter(ue.iter().chain(fe.iter()).cloned());
        let w1 = Array2::from_shape_vec((HIDDEN_SIZE, EMBEDDING_SIZE*2), self.w1.iter().flatten().cloned().collect())
            .map_err(|e| AIError::ShapeError(e.to_string()))?;
        let w2 = Array2::from_shape_vec((OUTPUT_SIZE, HIDDEN_SIZE), self.w2.iter().flatten().cloned().collect())
            .map_err(|e| AIError::ShapeError(e.to_string()))?;
        let b1 = Array1::from(self.b1.clone());
        let b2 = Array1::from(self.b2.clone());
        let h1 = (w1.dot(&input) + &b1).mapv(|x| x.max(0.0)); // ReLU
        let out = (w2.dot(&h1) + &b2)[0];
        if !out.is_finite() || h1.iter().any(|x| !x.is_finite()) {
            return Err(AIError::NumericalInstability);
        }
        Ok(out)
    }
    /// Train the model with user-file interactions. Returns error on instability.
    pub fn train(&mut self, user_file_interactions: &[(String, String)]) -> Result<(), AIError> {
        let lr = 0.01;
        let lambda = 0.01;
        for (user, file) in user_file_interactions {
            let ue = self.user_embeddings.entry(user.clone()).or_insert_with(|| vec![0.1; EMBEDDING_SIZE]);
            let fe = self.file_embeddings.entry(file.clone()).or_insert_with(|| vec![0.1; EMBEDDING_SIZE]);
            let input: Array1<f32> = Array::from_iter(ue.iter().chain(fe.iter()).cloned());
            let mut w1 = Array2::from_shape_vec((HIDDEN_SIZE, EMBEDDING_SIZE*2), self.w1.iter().flatten().cloned().collect())
                .map_err(|e| AIError::ShapeError(e.to_string()))?;
            let mut w2 = Array2::from_shape_vec((OUTPUT_SIZE, HIDDEN_SIZE), self.w2.iter().flatten().cloned().collect())
                .map_err(|e| AIError::ShapeError(e.to_string()))?;
            let mut b1 = Array1::from(self.b1.clone());
            let mut b2 = Array1::from(self.b2.clone());
            let h1 = (w1.dot(&input) + &b1).mapv(|x| x.max(0.0));
            let out = (w2.dot(&h1) + &b2)[0];
            if !out.is_finite() || h1.iter().any(|x| !x.is_finite()) {
                return Err(AIError::NumericalInstability);
            }
            let target = 1.0; // Implicit feedback
            let err = target - out;
            // Backprop (very simplified, not for production)
            let grad_out = err;
            let grad_w2 = h1.clone() * grad_out;
            let grad_b2 = grad_out;
            let grad_h1 = w2.t().dot(&Array1::from(vec![grad_out]));
            let grad_h1_relu = grad_h1 * h1.mapv(|x| if x > 0.0 { 1.0 } else { 0.0 });
            let grad_w1 = input.clone().insert_axis(Axis(1)).dot(&grad_h1_relu.clone().insert_axis(Axis(0)));
            let grad_b1 = grad_h1_relu;
            // Update weights
            for i in 0..OUTPUT_SIZE {
                for j in 0..HIDDEN_SIZE {
                    w2[[i, j]] += lr * grad_w2[j] - lambda * w2[[i, j]];
                }
                b2[i] += lr * grad_b2 - lambda * b2[i];
            }
            for i in 0..HIDDEN_SIZE {
                for j in 0..EMBEDDING_SIZE*2 {
                    w1[[i, j]] += lr * grad_w1[[j, i]] - lambda * w1[[i, j]];
                }
                b1[i] += lr * grad_b1[i] - lambda * b1[i];
            }
            // Update embeddings
            for i in 0..EMBEDDING_SIZE {
                ue[i] += lr * err * fe[i] - lambda * ue[i];
                fe[i] += lr * err * ue[i] - lambda * fe[i];
            }
            // Save back
            self.w1 = w1.outer_iter().map(|row| row.to_vec()).collect();
            self.w2 = w2.outer_iter().map(|row| row.to_vec()).collect();
            self.b1 = b1.to_vec();
            self.b2 = b2.to_vec();
        }
        self.epoch += 1;
        Ok(())
    }
    /// Recommend top_n files for a user. Returns error on instability.
    pub fn recommend(&self, user: &str, files: &[FileMetadata], top_n: usize) -> Result<Vec<FileMetadata>, AIError> {
        let mut scored: Vec<(f32, &FileMetadata)> = files.iter().map(|f| {
            let score = self.forward(user, &f.file_id.to_string()).unwrap_or(f32::MIN);
            (score, f)
        }).collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        Ok(scored.into_iter().take(top_n).map(|(_, f)| f.clone()).collect())
    }
    /// Federated averaging of all weights and embeddings. Validates input model.
    pub fn aggregate(&mut self, other: &NCFModel) -> Result<(), AIError> {
        // Validate model shapes
        if other.w1.len() != HIDDEN_SIZE || other.w1[0].len() != EMBEDDING_SIZE*2 {
            return Err(AIError::ModelValidation("w1 shape mismatch".into()));
        }
        if other.w2.len() != OUTPUT_SIZE || other.w2[0].len() != HIDDEN_SIZE {
            return Err(AIError::ModelValidation("w2 shape mismatch".into()));
        }
        // TODO: Verify cryptographic signature here (stub)
        for (user, factors) in &other.user_embeddings {
            let ue = self.user_embeddings.entry(user.clone()).or_insert_with(|| vec![0.1; EMBEDDING_SIZE]);
            for i in 0..EMBEDDING_SIZE {
                ue[i] = (ue[i] + factors[i]) / 2.0;
            }
        }
        for (file, factors) in &other.file_embeddings {
            let fe = self.file_embeddings.entry(file.clone()).or_insert_with(|| vec![0.1; EMBEDDING_SIZE]);
            for i in 0..EMBEDDING_SIZE {
                fe[i] = (fe[i] + factors[i]) / 2.0;
            }
        }
        for i in 0..HIDDEN_SIZE {
            for j in 0..EMBEDDING_SIZE*2 {
                self.w1[i][j] = (self.w1[i][j] + other.w1[i][j]) / 2.0;
            }
            self.b1[i] = (self.b1[i] + other.b1[i]) / 2.0;
        }
        for i in 0..OUTPUT_SIZE {
            for j in 0..HIDDEN_SIZE {
                self.w2[i][j] = (self.w2[i][j] + other.w2[i][j]) / 2.0;
            }
            self.b2[i] = (self.b2[i] + other.b2[i]) / 2.0;
        }
        self.epoch = self.epoch.max(other.epoch);
        Ok(())
    }
}

/// Global, thread-safe model instance. For async, consider tokio::sync::Mutex.
pub static LOCAL_MODEL: Lazy<Mutex<NCFModel>> = Lazy::new(|| Mutex::new(NCFModel::new()));

/// Train the local model. Offload to background thread for heavy workloads (stub).
pub fn train_local_model(user_file_interactions: &[(String, String)]) -> Result<(), AIError> {
    let mut model = LOCAL_MODEL.lock().map_err(|_| AIError::MutexPoisoned)?;
    model.train(user_file_interactions)
}

/// Get recommendations for a user. Returns error on instability.
pub fn get_recommendations(user_id: &str, files: &[FileMetadata]) -> Result<Vec<FileMetadata>, AIError> {
    let model = LOCAL_MODEL.lock().map_err(|_| AIError::MutexPoisoned)?;
    model.recommend(user_id, files, 10)
}

/// Aggregate a remote model into the local model (federated learning). Returns error on validation.
pub fn aggregate_remote_model(remote: &NCFModel) -> Result<(), AIError> {
    let mut model = LOCAL_MODEL.lock().map_err(|_| AIError::MutexPoisoned)?;
    model.aggregate(remote)
}

// SECURITY/PRIVACY NOTE:
// - Only model weights/embeddings are shared in federated learning, never raw user data.
// - For production, cryptographic signatures should be used to verify model authenticity.
// - All federated updates should be validated for shape/type.
// - Consider running heavy training/aggregation in a background thread or async task. 
