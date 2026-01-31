//! Concrete NER engine implementation using GLiNER via gline-rs.
//!
//! This module is only compiled when the `gliner` feature is enabled.

use std::path::Path;
use std::sync::Mutex;

use gliner::model::GLiNER;
use gliner::model::input::text::TextInput;
use gliner::model::params::Parameters;
use gliner::model::pipeline::span::SpanMode;
use orp::params::RuntimeParameters;

use super::ner::{NerConfig, NerEngine, NerOutput, NerSpan};

/// GLiNER-based NER engine using span mode.
///
/// Wraps a `GLiNER<SpanMode>` model behind a Mutex since inference
/// requires `&mut self`.
pub struct GlinerEngine {
    model: Mutex<GLiNER<SpanMode>>,
    threshold: f32,
}

impl GlinerEngine {
    /// Create a new GlinerEngine from model and tokenizer file paths.
    pub fn new(config: &NerConfig) -> Result<Self, String> {
        let params = Parameters::default().with_threshold(config.threshold);
        let runtime_params = RuntimeParameters::default();
        let model = GLiNER::<SpanMode>::new(
            params,
            runtime_params,
            Path::new(&config.tokenizer_path),
            Path::new(&config.model_path),
        )
        .map_err(|e| format!("Failed to load GLiNER model: {e}"))?;

        Ok(Self {
            model: Mutex::new(model),
            threshold: config.threshold,
        })
    }
}

impl NerEngine for GlinerEngine {
    fn extract(&self, texts: &[&str], entity_labels: &[&str]) -> Result<NerOutput, String> {
        let input =
            TextInput::from_str(texts, entity_labels).map_err(|e| format!("Input error: {e}"))?;

        let mut model = self
            .model
            .lock()
            .map_err(|e| format!("Lock poisoned: {e}"))?;

        let output = model
            .inference(input)
            .map_err(|e| format!("Inference error: {e}"))?;

        let entities: Vec<NerSpan> = output
            .spans
            .iter()
            .flat_map(|batch| batch.iter())
            .filter(|span| span.probability() >= self.threshold)
            .map(|span| NerSpan {
                text: span.text().to_string(),
                label: span.class().to_string(),
                score: span.probability(),
            })
            .collect();

        Ok(NerOutput {
            entities,
            relations: vec![],
        })
    }
}
