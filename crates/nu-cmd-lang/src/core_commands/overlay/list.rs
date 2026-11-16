use nu_engine::command_prelude::*;

#[derive(Clone)]
pub struct OverlayList;

impl Command for OverlayList {
    fn name(&self) -> &str {
        "overlay list"
    }

    fn description(&self) -> &str {
        "List all overlays and their active status."
    }

    fn signature(&self) -> nu_protocol::Signature {
        Signature::build("overlay list")
            .category(Category::Core)
            .input_output_types(vec![(
                Type::Nothing,
                Type::Table(
                    vec![("name".into(), Type::String), ("active".into(), Type::Bool)].into(),
                ),
            )])
    }

    fn extra_description(&self) -> &str {
        "Hidden overlays are listed first, followed by active overlays in activation order. The last overlay is the topmost, most recently activated overlay."
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        _input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        let mut seen = std::collections::HashSet::new();
        let hidden = engine_state
            .scope
            .overlays
            .iter()
            .map(|(name, _)| String::from_utf8_lossy(name).into_owned())
            .chain(stack.hidden_overlays.iter().cloned())
            .filter(|name| !stack.is_overlay_active(name) && seen.insert(name.clone()))
            .map(|name| {
                Value::record(
                    record! {
                        "name" => Value::string(name, call.head),
                        "active" => Value::bool(false, call.head),
                    },
                    call.head,
                )
            });
        let active = stack.active_overlays.iter().map(|name| {
            Value::record(
                record! {
                    "name" => Value::string(name, call.head),
                    "active" => Value::bool(true, call.head),
                },
                call.head,
            )
        });

        Ok(Value::list(hidden.chain(active).collect(), call.head).into_pipeline_data())
    }

    fn examples(&self) -> Vec<Example> {
        vec![Example {
            description: "Get the last activated overlay",
            example: r#"module spam { export def foo [] { "foo" } }
    overlay use spam
    overlay list | last | get name"#,
            result: Some(Value::test_string("spam")),
        }]
    }
}
