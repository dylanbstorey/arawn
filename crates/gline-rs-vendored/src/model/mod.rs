//! The core of `gline-rs`: everything about pre-/post-processing, and inferencing

pub mod input;
pub mod output;
pub mod params;
pub mod pipeline;

use crate::util::result::Result;
use orp::model::Model;
use orp::pipeline::Pipeline;
use params::Parameters;

/// Basic GLiNER, to be parametrized by a specific pipeline (see implementations within the pipeline module)
///
/// This is just a convenience wrapper around a `Model`, a `Pipeline`, and some `Parameters`.
pub struct GLiNER<P> {
    params: Parameters,
    model: Model,
    pipeline: P,
}

impl<'a, P: Pipeline<'a, Parameters = Parameters>> GLiNER<P> {
    pub fn inference(&'a mut self, input: P::Input) -> Result<P::Output> {
        self.model.inference(input, &self.pipeline, &self.params)
    }
}
