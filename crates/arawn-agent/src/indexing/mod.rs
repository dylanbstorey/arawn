//! Session indexing pipeline: extraction, summarization, and memory storage.

pub mod extraction;
#[cfg(feature = "gliner")]
pub mod gliner;
pub mod indexer;
pub mod ner;
mod report;
pub mod summarization;
mod types;

pub use extraction::ExtractionPrompt;
#[cfg(feature = "gliner")]
pub use gliner::GlinerEngine;
pub use indexer::{Completer, IndexerConfig, SessionIndexer};
pub use ner::{NerConfig, NerEngine, NerExtraction, NerOutput, NerRelation, NerSpan};
pub use report::IndexReport;
pub use summarization::SummarizationPrompt;
pub use types::{ExtractedEntity, ExtractedFact, ExtractedRelationship, ExtractionResult};
