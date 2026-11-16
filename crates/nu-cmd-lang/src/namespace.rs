use nu_engine::{command_prelude::*, get_full_help};
#[derive(Clone)]
pub struct Attr;
impl Command for Attr {
    fn name(&self) -> &str {
        "attr"
    }
    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .category(Category::Core)
            .input_output_types(vec![(Type::Nothing, Type::String)])
    }
    fn description(&self) -> &str {
        "Commands in the attr namespace."
    }
    fn extra_description(&self) -> &str {
        "A subcommand is required. Using this command alone displays help."
    }
    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        _input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        Ok(Value::string(get_full_help(self, engine_state, stack), call.head).into_pipeline_data())
    }
}
#[derive(Clone)]
pub struct Error;
impl Command for Error {
    fn name(&self) -> &str {
        "error"
    }
    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .category(Category::Core)
            .input_output_types(vec![(Type::Nothing, Type::String)])
    }
    fn description(&self) -> &str {
        "Commands in the error namespace."
    }
    fn extra_description(&self) -> &str {
        "A subcommand is required. Using this command alone displays help."
    }
    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        _input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        Ok(Value::string(get_full_help(self, engine_state, stack), call.head).into_pipeline_data())
    }
}
