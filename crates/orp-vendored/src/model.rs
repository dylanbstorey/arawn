use crate::error::UnexpectedModelSchemaError;
use composable::Composable;
use ort::session::{builder::GraphOptimizationLevel, Session, SessionInputs, SessionOutputs};
use std::collections::HashSet;
use std::path::Path;

use super::params::RuntimeParameters;
use super::pipeline::Pipeline;
use super::Result;

/// A `Model` can load an ONNX model, and run it using the provided pipeline.
pub struct Model {
    session: Session,
}

impl Model {
    pub fn new<P: AsRef<Path>>(model_path: P, params: RuntimeParameters) -> Result<Self> {
        let session = Session::builder()?
            .with_intra_threads(params.threads())?
            .with_execution_providers(params.into_execution_providers())?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .commit_from_file(model_path)?;

        Ok(Self { session })
    }

    pub fn new_from_bytes(model_bytes: &[u8], params: RuntimeParameters) -> Result<Self> {
        let session = Session::builder()?
            .with_intra_threads(params.threads())?
            .with_execution_providers(params.into_execution_providers())?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .commit_from_memory(model_bytes)?;

        Ok(Self { session })
    }

    /// Perform inferences using the provided pipeline and parameters
    pub fn inference<'a, P: Pipeline<'a>>(
        &'a mut self,
        input: P::Input,
        pipeline: &P,
        params: &P::Parameters,
    ) -> Result<P::Output> {
        // check schema
        self.check_schema(pipeline, params)?;
        // pre-process
        let (input, context) = pipeline.pre_processor(params).apply(input)?;
        // inference
        let output = self.run(input)?;
        // post-process
        let output = pipeline.post_processor(params).apply((output, context))?;
        // ok
        Ok(output)
    }

    pub fn to_composable<'a, P: Pipeline<'a>>(
        &'a mut self,
        pipeline: &'a P,
        params: &'a P::Parameters,
    ) -> impl Composable<P::Input, P::Output> {
        ComposableModel::new(self, pipeline, params)
    }

    /// Writes various model properties from metadata and input/output tensors
    pub fn inspect<W: std::io::Write>(&self, mut writer: W) -> Result<()> {
        let metadata = self.session.metadata()?;
        writeln!(writer, "NAME: {}", metadata.name().unwrap_or_default())?;
        writeln!(
            writer,
            "PRODUCER: {}",
            metadata.producer().unwrap_or_default()
        )?;
        writeln!(
            writer,
            "VERSION: {}",
            metadata.version().unwrap_or_default()
        )?;
        writeln!(writer, "INPUTS:")?;
        for input in self.session.inputs() {
            writeln!(writer, "\t{}: {:?}", input.name(), input.dtype())?;
        }
        writeln!(writer, "OUTPUTS:")?;
        for output in self.session.outputs() {
            writeln!(writer, "\t{}: {:?}", output.name(), output.dtype())?;
        }
        Ok(())
    }

    /// Check model schema wrt. pipeline expectations
    fn check_schema<'a, P: Pipeline<'a>>(
        &self,
        pipeline: &P,
        params: &P::Parameters,
    ) -> Result<()> {
        if let Some(expected_inputs) = pipeline.expected_inputs(params) {
            // inputs should be exactly the same sets
            let expected_inputs = &expected_inputs.collect();
            let actual_inputs: HashSet<_> =
                self.session.inputs().iter().map(|i| i.name()).collect();
            if !actual_inputs.eq(expected_inputs) {
                return UnexpectedModelSchemaError::new_for_input(expected_inputs, &actual_inputs)
                    .into_err();
            }
        }
        if let Some(expected_outputs) = pipeline.expected_outputs(params) {
            // for outputs, we just check that the expected ones are present (but having others is ok)
            let expected_outputs = &expected_outputs.collect();
            let actual_outputs: HashSet<_> =
                self.session.outputs().iter().map(|i| i.name()).collect();
            if !actual_outputs.is_superset(&expected_outputs) {
                return UnexpectedModelSchemaError::new_for_output(
                    expected_outputs,
                    &actual_outputs,
                )
                .into_err();
            }
        }
        Ok(())
    }

    fn run(&mut self, input: SessionInputs<'_, '_>) -> Result<SessionOutputs<'_>> {
        Ok(self.session.run(input)?)
    }
}

/// References a model, a pipeline and some parameters to implement `Composable`
struct ComposableModel<'a, P: Pipeline<'a>> {
    model: &'a mut Model,
    pipeline: &'a P,
    params: &'a P::Parameters,
}

impl<'a, P: Pipeline<'a>> ComposableModel<'a, P> {
    pub fn new(model: &'a mut Model, pipeline: &'a P, params: &'a P::Parameters) -> Self {
        Self {
            model,
            pipeline,
            params,
        }
    }
}

impl<'a, P: Pipeline<'a>> Composable<P::Input, P::Output> for ComposableModel<'a, P> {
    fn apply(&self, _input: P::Input) -> Result<P::Output> {
        // NOTE: Cannot call inference(&mut self) from &self context.
        // ComposableModel is rarely used; callers should use Model::inference directly.
        unimplemented!("ComposableModel requires mutable access; use Model::inference() directly")
    }
}
