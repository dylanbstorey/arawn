//! Embeddings support for semantic search and memory.
//!
//! This module provides the [`Embedder`] trait and implementations for
//! generating vector embeddings from text. Embeddings are used for:
//! - Semantic search in the memory store
//! - Finding similar content
//! - Context retrieval for conversations
//!
//! # Implementations
//!
//! - [`MockEmbedder`]: Returns deterministic embeddings for testing
//! - [`LocalEmbedder`]: Uses ONNX Runtime for local inference (requires `local-embeddings` feature)
//! - [`OpenAiEmbedder`]: Uses OpenAI's embeddings API

use async_trait::async_trait;
use std::sync::Arc;

use crate::error::Result;

// ─────────────────────────────────────────────────────────────────────────────
// Embedder Trait
// ─────────────────────────────────────────────────────────────────────────────

/// Trait for generating text embeddings.
///
/// Embedders convert text into dense vector representations that capture
/// semantic meaning, enabling similarity search and retrieval.
#[async_trait]
pub trait Embedder: Send + Sync {
    /// Generate an embedding for a single text.
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;

    /// Generate embeddings for multiple texts in a batch.
    ///
    /// Default implementation calls `embed` for each text sequentially.
    /// Implementations may override for more efficient batching.
    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::with_capacity(texts.len());
        for text in texts {
            results.push(self.embed(text).await?);
        }
        Ok(results)
    }

    /// Get the dimensionality of embeddings produced by this embedder.
    fn dimensions(&self) -> usize;

    /// Get the name of this embedder.
    fn name(&self) -> &str;
}

/// A shared embedder that can be used across threads.
pub type SharedEmbedder = Arc<dyn Embedder>;

// ─────────────────────────────────────────────────────────────────────────────
// Mock Embedder
// ─────────────────────────────────────────────────────────────────────────────

/// A mock embedder for testing purposes.
///
/// Generates deterministic embeddings based on text content, useful for
/// testing similarity search and memory retrieval without external dependencies.
#[derive(Debug, Clone)]
pub struct MockEmbedder {
    dimensions: usize,
}

impl MockEmbedder {
    /// Create a new mock embedder with the specified dimensions.
    pub fn new(dimensions: usize) -> Self {
        Self { dimensions }
    }

    /// Create a mock embedder with 384 dimensions (same as all-MiniLM-L6-v2).
    pub fn default_dimensions() -> Self {
        Self::new(384)
    }
}

impl Default for MockEmbedder {
    fn default() -> Self {
        Self::default_dimensions()
    }
}

#[async_trait]
impl Embedder for MockEmbedder {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // Generate a deterministic embedding based on text hash
        // This ensures the same text always produces the same embedding
        let hash = simple_hash(text);
        let mut embedding = vec![0.0f32; self.dimensions];

        // Fill embedding with pseudo-random values based on hash
        let mut state = hash;
        for i in 0..self.dimensions {
            state = state.wrapping_mul(1103515245).wrapping_add(12345);
            embedding[i] = ((state >> 16) as f32 / 32768.0) - 1.0;
        }

        // Normalize to unit length
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in &mut embedding {
                *x /= norm;
            }
        }

        Ok(embedding)
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }

    fn name(&self) -> &str {
        "mock"
    }
}

/// Simple hash function for deterministic embedding generation.
fn simple_hash(s: &str) -> u64 {
    let mut hash: u64 = 5381;
    for byte in s.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
    }
    hash
}

// ─────────────────────────────────────────────────────────────────────────────
// OpenAI Embedder
// ─────────────────────────────────────────────────────────────────────────────

use reqwest::Client;
use std::time::Duration;

/// Configuration for OpenAI embeddings.
#[derive(Debug, Clone)]
pub struct OpenAiEmbedderConfig {
    /// API key for authentication.
    pub api_key: String,
    /// Base URL for the API.
    pub base_url: String,
    /// Model to use for embeddings.
    pub model: String,
    /// Request timeout.
    pub timeout: Duration,
}

impl OpenAiEmbedderConfig {
    /// Create a new config with the given API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: "https://api.openai.com/v1".to_string(),
            model: "text-embedding-3-small".to_string(),
            timeout: Duration::from_secs(60),
        }
    }

    /// Create config from environment variable.
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("OPENAI_API_KEY").map_err(|_| {
            crate::error::LlmError::Config(
                "OPENAI_API_KEY environment variable not set".to_string(),
            )
        })?;
        Ok(Self::new(api_key))
    }

    /// Set a custom base URL.
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Set the model to use.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }
}

/// OpenAI embeddings API client.
pub struct OpenAiEmbedder {
    client: Client,
    config: OpenAiEmbedderConfig,
    dimensions: usize,
}

impl OpenAiEmbedder {
    /// Create a new OpenAI embedder.
    pub fn new(config: OpenAiEmbedderConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(|e| {
                crate::error::LlmError::Internal(format!("Failed to create HTTP client: {}", e))
            })?;

        // Determine dimensions based on model
        let dimensions = match config.model.as_str() {
            "text-embedding-3-small" => 1536,
            "text-embedding-3-large" => 3072,
            "text-embedding-ada-002" => 1536,
            _ => 1536, // Default
        };

        Ok(Self {
            client,
            config,
            dimensions,
        })
    }

    /// Create from environment configuration.
    pub fn from_env() -> Result<Self> {
        Self::new(OpenAiEmbedderConfig::from_env()?)
    }

    fn embeddings_url(&self) -> String {
        format!("{}/embeddings", self.config.base_url)
    }
}

#[async_trait]
impl Embedder for OpenAiEmbedder {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let results = self.embed_batch(&[text]).await?;
        results
            .into_iter()
            .next()
            .ok_or_else(|| crate::error::LlmError::Internal("No embedding returned".to_string()))
    }

    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let request = EmbeddingRequest {
            model: self.config.model.clone(),
            input: texts.iter().map(|s| s.to_string()).collect(),
        };

        let response = self
            .client
            .post(self.embeddings_url())
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(crate::error::LlmError::Backend(format!(
                "Embedding request failed: HTTP {} - {}",
                status, body
            )));
        }

        let result: EmbeddingResponse = response.json().await.map_err(|e| {
            crate::error::LlmError::Serialization(format!("Failed to parse response: {}", e))
        })?;

        // Sort by index to ensure correct order
        let mut embeddings: Vec<_> = result.data.into_iter().collect();
        embeddings.sort_by_key(|e| e.index);

        Ok(embeddings.into_iter().map(|e| e.embedding).collect())
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }

    fn name(&self) -> &str {
        "openai"
    }
}

#[derive(Debug, serde::Serialize)]
struct EmbeddingRequest {
    model: String,
    input: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Debug, serde::Deserialize)]
struct EmbeddingData {
    index: usize,
    embedding: Vec<f32>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Local Embedder (ONNX Runtime)
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "local-embeddings")]
pub mod local {
    //! Local embeddings using ONNX Runtime.
    //!
    //! This module requires the `local-embeddings` feature to be enabled.

    use super::*;
    use ndarray::Array2;
    use ort::{GraphOptimizationLevel, Session};
    use std::path::Path;
    use tokenizers::Tokenizer;

    /// Local embedder using ONNX Runtime.
    ///
    /// This embedder runs inference locally using an ONNX model, enabling
    /// offline operation and avoiding API costs.
    pub struct LocalEmbedder {
        session: Session,
        tokenizer: Tokenizer,
        dimensions: usize,
        name: String,
    }

    impl LocalEmbedder {
        /// Load a local embedder from model files.
        ///
        /// # Arguments
        /// * `model_path` - Path to the ONNX model file
        /// * `tokenizer_path` - Path to the tokenizer.json file
        /// * `dimensions` - Output embedding dimensions
        pub fn load(
            model_path: impl AsRef<Path>,
            tokenizer_path: impl AsRef<Path>,
            dimensions: usize,
        ) -> Result<Self> {
            let session = Session::builder()
                .map_err(|e| {
                    crate::error::LlmError::Internal(format!(
                        "Failed to create ONNX session: {}",
                        e
                    ))
                })?
                .with_optimization_level(GraphOptimizationLevel::Level3)
                .map_err(|e| {
                    crate::error::LlmError::Internal(format!(
                        "Failed to set optimization level: {}",
                        e
                    ))
                })?
                .commit_from_file(model_path.as_ref())
                .map_err(|e| {
                    crate::error::LlmError::Internal(format!(
                        "Failed to load ONNX model from {:?}: {}",
                        model_path.as_ref(),
                        e
                    ))
                })?;

            let tokenizer = Tokenizer::from_file(tokenizer_path.as_ref()).map_err(|e| {
                crate::error::LlmError::Internal(format!(
                    "Failed to load tokenizer from {:?}: {}",
                    tokenizer_path.as_ref(),
                    e
                ))
            })?;

            Ok(Self {
                session,
                tokenizer,
                dimensions,
                name: "local".to_string(),
            })
        }
    }

    #[async_trait]
    impl Embedder for LocalEmbedder {
        async fn embed(&self, text: &str) -> Result<Vec<f32>> {
            let results = self.embed_batch(&[text]).await?;
            results.into_iter().next().ok_or_else(|| {
                crate::error::LlmError::Internal("No embedding returned".to_string())
            })
        }

        async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
            if texts.is_empty() {
                return Ok(Vec::new());
            }

            // Tokenize all inputs
            let encodings: Vec<_> = texts
                .iter()
                .map(|text| {
                    self.tokenizer.encode(*text, true).map_err(|e| {
                        crate::error::LlmError::Internal(format!("Tokenization failed: {}", e))
                    })
                })
                .collect::<Result<Vec<_>>>()?;

            // Process in chunks to avoid OOM on large batches
            let chunk_size = 32;
            let mut all_results = Vec::with_capacity(texts.len());

            for chunk in encodings.chunks(chunk_size) {
                let batch_results = self.run_batch(chunk)?;
                all_results.extend(batch_results);
            }

            Ok(all_results)
        }

        fn dimensions(&self) -> usize {
            self.dimensions
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    impl LocalEmbedder {
        /// Run ONNX inference on a batch of encodings.
        ///
        /// Pads all sequences to the same length and runs a single
        /// `session.run()` call with batched tensors.
        fn run_batch(&self, encodings: &[tokenizers::Encoding]) -> Result<Vec<Vec<f32>>> {
            let batch_size = encodings.len();

            // Find max sequence length for padding
            let max_len = encodings
                .iter()
                .map(|e| e.get_ids().len())
                .max()
                .unwrap_or(0);

            // Build padded 2D arrays: (batch_size, max_len)
            let mut input_ids_flat = vec![0i64; batch_size * max_len];
            let mut attention_mask_flat = vec![0i64; batch_size * max_len];
            let mut token_type_ids_flat = vec![0i64; batch_size * max_len];

            for (i, enc) in encodings.iter().enumerate() {
                let ids = enc.get_ids();
                let mask = enc.get_attention_mask();
                let types = enc.get_type_ids();
                let seq_len = ids.len();
                let offset = i * max_len;

                for j in 0..seq_len {
                    input_ids_flat[offset + j] = ids[j] as i64;
                    attention_mask_flat[offset + j] = mask[j] as i64;
                    token_type_ids_flat[offset + j] = types[j] as i64;
                }
                // Rest stays zero-padded
            }

            let input_ids_array = Array2::from_shape_vec((batch_size, max_len), input_ids_flat)
                .map_err(|e| crate::error::LlmError::Internal(format!("Array error: {}", e)))?;
            let attention_mask_array =
                Array2::from_shape_vec((batch_size, max_len), attention_mask_flat.clone())
                    .map_err(|e| crate::error::LlmError::Internal(format!("Array error: {}", e)))?;
            let token_type_ids_array =
                Array2::from_shape_vec((batch_size, max_len), token_type_ids_flat)
                    .map_err(|e| crate::error::LlmError::Internal(format!("Array error: {}", e)))?;

            let outputs = self
                .session
                .run(
                    ort::inputs![
                        "input_ids" => input_ids_array.view(),
                        "attention_mask" => attention_mask_array.view(),
                        "token_type_ids" => token_type_ids_array.view(),
                    ]
                    .map_err(|e| crate::error::LlmError::Internal(format!("Input error: {}", e)))?,
                )
                .map_err(|e| {
                    crate::error::LlmError::Internal(format!("ONNX inference failed: {}", e))
                })?;

            let embeddings = outputs[0].try_extract_tensor::<f32>().map_err(|e| {
                crate::error::LlmError::Internal(format!("Output extraction failed: {}", e))
            })?;

            let embeddings_array = embeddings.view().to_owned();
            let shape = embeddings_array.shape();
            // shape is (batch_size, seq_len, hidden_dim) → reshape to (batch_size, seq_len * hidden_dim)
            // Actually for mean pooling we need (batch_size, seq_len, hidden_dim)
            // then pool over seq_len axis per sample

            let hidden_dim = shape[2];
            let seq_len_out = shape[1];

            let mut results = Vec::with_capacity(batch_size);

            for i in 0..batch_size {
                // Extract this sample's embeddings: (seq_len, hidden_dim)
                let sample = embeddings_array
                    .slice(ndarray::s![i..i + 1, .., ..])
                    .to_owned();
                let sample_2d = sample
                    .into_shape_with_order((seq_len_out, hidden_dim))
                    .map_err(|e| {
                        crate::error::LlmError::Internal(format!("Reshape error: {}", e))
                    })?;

                // This sample's attention mask
                let mask_slice = &attention_mask_flat[i * max_len..(i + 1) * max_len];

                // Mean pooling with attention mask
                let mut sum = vec![0.0f32; hidden_dim];
                let mut count = 0.0f32;
                for (j, &mask_val) in mask_slice.iter().enumerate() {
                    if mask_val > 0 {
                        let row = sample_2d.row(j);
                        for (k, &v) in row.iter().enumerate() {
                            sum[k] += v;
                        }
                        count += 1.0;
                    }
                }

                if count > 0.0 {
                    for v in &mut sum {
                        *v /= count;
                    }
                }

                // L2 normalize
                let norm: f32 = sum.iter().map(|x| x * x).sum::<f32>().sqrt();
                if norm > 1e-9 {
                    for v in &mut sum {
                        *v /= norm;
                    }
                }

                results.push(sum);
            }

            Ok(results)
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Embedder Factory
// ─────────────────────────────────────────────────────────────────────────────

/// Configuration for building an embedder from application config.
///
/// This is a provider-agnostic struct that `start.rs` populates from
/// `EmbeddingConfig`. It avoids a dependency from arawn-llm → arawn-config.
#[derive(Debug, Clone)]
pub struct EmbedderSpec {
    /// Provider name: "local", "openai", or "mock".
    pub provider: String,
    /// OpenAI API key (required for "openai" provider).
    pub openai_api_key: Option<String>,
    /// OpenAI model name.
    pub openai_model: Option<String>,
    /// OpenAI base URL override.
    pub openai_base_url: Option<String>,
    /// Local ONNX model path.
    pub local_model_path: Option<std::path::PathBuf>,
    /// Local tokenizer.json path.
    pub local_tokenizer_path: Option<std::path::PathBuf>,
    /// Requested dimensions (for providers that support it).
    pub dimensions: Option<usize>,
}

/// Build a `SharedEmbedder` from a spec.
///
/// Falls back to `MockEmbedder` if the requested provider is unavailable
/// (e.g., "local" requested but `local-embeddings` feature disabled).
pub fn build_embedder(spec: &EmbedderSpec) -> Result<SharedEmbedder> {
    match spec.provider.as_str() {
        "openai" => {
            let api_key = spec.openai_api_key.as_deref().ok_or_else(|| {
                crate::error::LlmError::Config(
                    "OpenAI embedding provider requires an API key. \
                     Set OPENAI_API_KEY or configure [embedding.openai] api_key."
                        .to_string(),
                )
            })?;
            let mut config = OpenAiEmbedderConfig::new(api_key);
            if let Some(ref model) = spec.openai_model {
                config = config.with_model(model);
            }
            if let Some(ref base_url) = spec.openai_base_url {
                config = config.with_base_url(base_url);
            }
            Ok(Arc::new(OpenAiEmbedder::new(config)?))
        }
        #[cfg(feature = "local-embeddings")]
        "local" => {
            let dims = spec.dimensions.unwrap_or(384);
            match (&spec.local_model_path, &spec.local_tokenizer_path) {
                (Some(model_path), Some(tokenizer_path)) => {
                    let embedder = local::LocalEmbedder::load(model_path, tokenizer_path, dims)?;
                    Ok(Arc::new(embedder))
                }
                _ => {
                    // Try default model location
                    let default_dir = default_local_model_dir();
                    if let Some(dir) = default_dir {
                        let model_path = dir.join("model.onnx");
                        let tokenizer_path = dir.join("tokenizer.json");
                        if model_path.exists() && tokenizer_path.exists() {
                            let embedder =
                                local::LocalEmbedder::load(&model_path, &tokenizer_path, dims)?;
                            return Ok(Arc::new(embedder));
                        }
                    }
                    tracing::warn!(
                        "Local embedding model not found. Falling back to mock embedder. \
                         Download all-MiniLM-L6-v2 ONNX model to ~/.local/share/arawn/models/embeddings/"
                    );
                    Ok(Arc::new(MockEmbedder::new(dims)))
                }
            }
        }
        #[cfg(not(feature = "local-embeddings"))]
        "local" => {
            tracing::warn!(
                "Local embeddings requested but 'local-embeddings' feature is not enabled. \
                 Falling back to mock embedder."
            );
            let dims = spec.dimensions.unwrap_or(384);
            Ok(Arc::new(MockEmbedder::new(dims)))
        }
        "mock" => {
            let dims = spec.dimensions.unwrap_or(384);
            Ok(Arc::new(MockEmbedder::new(dims)))
        }
        other => Err(crate::error::LlmError::Config(format!(
            "Unknown embedding provider '{}'. Valid: local, openai, mock",
            other
        ))),
    }
}

/// Default directory for local embedding model files.
fn default_local_model_dir() -> Option<std::path::PathBuf> {
    dirs::data_dir().map(|d| d.join("arawn").join("models").join("embeddings"))
}

// ─────────────────────────────────────────────────────────────────────────────
// Utility Functions
// ─────────────────────────────────────────────────────────────────────────────

/// Calculate cosine similarity between two embeddings.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a > 0.0 && norm_b > 0.0 {
        dot / (norm_a * norm_b)
    } else {
        0.0
    }
}

/// Calculate Euclidean distance between two embeddings.
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return f32::MAX;
    }

    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f32>()
        .sqrt()
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_embedder() {
        let embedder = MockEmbedder::default();
        assert_eq!(embedder.dimensions(), 384);
        assert_eq!(embedder.name(), "mock");

        let embedding = embedder.embed("hello world").await.unwrap();
        assert_eq!(embedding.len(), 384);

        // Check normalization (should be unit length)
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_mock_embedder_deterministic() {
        let embedder = MockEmbedder::default();

        let e1 = embedder.embed("test text").await.unwrap();
        let e2 = embedder.embed("test text").await.unwrap();

        // Same text should produce same embedding
        assert_eq!(e1, e2);
    }

    #[tokio::test]
    async fn test_mock_embedder_different_texts() {
        let embedder = MockEmbedder::default();

        let e1 = embedder.embed("hello").await.unwrap();
        let e2 = embedder.embed("world").await.unwrap();

        // Different texts should produce different embeddings
        assert_ne!(e1, e2);
    }

    #[tokio::test]
    async fn test_embed_batch() {
        let embedder = MockEmbedder::default();

        let texts = vec!["one", "two", "three"];
        let embeddings = embedder.embed_batch(&texts).await.unwrap();

        assert_eq!(embeddings.len(), 3);
        for emb in &embeddings {
            assert_eq!(emb.len(), 384);
        }
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let c = vec![0.0, 1.0, 0.0];
        assert!(cosine_similarity(&a, &c).abs() < 0.001);

        let d = vec![-1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &d) + 1.0).abs() < 0.001);
    }

    #[test]
    fn test_euclidean_distance() {
        let a = vec![0.0, 0.0, 0.0];
        let b = vec![3.0, 4.0, 0.0];
        assert!((euclidean_distance(&a, &b) - 5.0).abs() < 0.001);

        let c = vec![0.0, 0.0, 0.0];
        assert!(euclidean_distance(&a, &c).abs() < 0.001);
    }

    #[test]
    fn test_openai_embedder_config() {
        let config = OpenAiEmbedderConfig::new("test-key");
        assert_eq!(config.api_key, "test-key");
        assert_eq!(config.model, "text-embedding-3-small");
    }

    #[test]
    fn test_openai_embedder_config_builder() {
        let config = OpenAiEmbedderConfig::new("key")
            .with_base_url("http://custom.api")
            .with_model("text-embedding-ada-002");

        assert_eq!(config.base_url, "http://custom.api");
        assert_eq!(config.model, "text-embedding-ada-002");
    }
}
