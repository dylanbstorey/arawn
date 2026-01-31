//! Encapsulation of raw tensor outputs

use crate::model::pipeline::context::EntityContext;
use crate::util::result::Result;
use composable::Composable;
use ort::session::SessionOutputs;

/// Represents the raw tensor output of the inference step
pub struct TensorOutput<'a> {
    pub context: EntityContext,
    pub tensors: SessionOutputs<'a>,
}

impl<'a> TensorOutput<'a> {
    pub fn from(tensors: SessionOutputs<'a>, context: EntityContext) -> Self {
        Self { context, tensors }
    }
}

/// Composable: (SessionOutput, TensorMeta) => TensorOutput
#[derive(Default)]
pub struct SessionOutputToTensors {}

impl<'a> Composable<(SessionOutputs<'a>, EntityContext), TensorOutput<'a>>
    for SessionOutputToTensors
{
    fn apply(&self, input: (SessionOutputs<'a>, EntityContext)) -> Result<TensorOutput<'a>> {
        Ok(TensorOutput::from(input.0, input.1))
    }
}
