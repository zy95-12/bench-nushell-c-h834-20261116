use nu_engine::{command_prelude::*, get_full_help};
#[derive(Clone)]
pub struct Detect;
impl Command for Detect {
    fn name(&self) -> &str {
        "detect"
    }
    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .category(Category::Core)
            .input_output_types(vec![(Type::Nothing, Type::String)])
    }
    fn description(&self) -> &str {
        "Commands in the detect namespace."
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
pub struct Query;
impl Command for Query {
    fn name(&self) -> &str {
        "query"
    }
    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .category(Category::Core)
            .input_output_types(vec![(Type::Nothing, Type::String)])
    }
    fn description(&self) -> &str {
        "Commands in the query namespace."
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
#[cfg(windows)]
#[derive(Clone)]
pub struct Registry;
#[cfg(windows)]
impl Command for Registry {
    fn name(&self) -> &str {
        "registry"
    }
    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .category(Category::Core)
            .input_output_types(vec![(Type::Nothing, Type::String)])
    }
    fn description(&self) -> &str {
        "Commands in the registry namespace."
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
