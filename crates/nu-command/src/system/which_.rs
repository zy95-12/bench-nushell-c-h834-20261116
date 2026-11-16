use nu_engine::{command_prelude::*, env};
use nu_protocol::engine::CommandType;
use std::{ffi::OsStr, path::Path};

#[derive(Clone)]
pub struct Which;

impl Command for Which {
    fn name(&self) -> &str {
        "which"
    }

    fn signature(&self) -> Signature {
        Signature::build("which")
            .input_output_types(vec![(Type::Nothing, Type::table())])
            .allow_variants_without_examples(true)
            .rest("applications", SyntaxShape::String, "Application(s).")
            .switch("all", "list all executables", Some('a'))
            .category(Category::System)
    }

    fn description(&self) -> &str {
        "Find a program, alias or command, or list all commands when no name is given."
    }

    fn search_terms(&self) -> Vec<&str> {
        vec![
            "find",
            "path",
            "location",
            "command",
            "whereis",     // linux binary to find binary locations in path
            "get-command", // powershell command to find commands and binaries in path
        ]
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        _input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        which(engine_state, stack, call)
    }

    fn examples(&self) -> Vec<Example> {
        vec![Example {
            description: "Find if the 'myapp' application is available",
            example: "which myapp",
            result: None,
        }]
    }
}

// Shortcut for creating an entry to the output table
fn entry(
    arg: impl Into<String>,
    path: impl Into<String>,
    cmd_type: CommandType,
    span: Span,
) -> Value {
    Value::record(
        record! {
            "command" => Value::string(arg, span),
            "path" => Value::string(path, span),
            "type" => Value::string(cmd_type.to_string(), span),
        },
        span,
    )
}

fn get_entry_in_commands(engine_state: &EngineState, name: &str, span: Span) -> Option<Value> {
    if let Some(decl_id) = engine_state.find_decl(name.as_bytes(), &[]) {
        let decl = engine_state.get_decl(decl_id);
        Some(entry(name, "", decl.command_type(), span))
    } else {
        None
    }
}

fn get_entries_in_nu(
    engine_state: &EngineState,
    name: &str,
    span: Span,
    skip_after_first_found: bool,
) -> Vec<Value> {
    let mut all_entries = vec![];

    if !all_entries.is_empty() && skip_after_first_found {
        return all_entries;
    }

    if let Some(ent) = get_entry_in_commands(engine_state, name, span) {
        all_entries.push(ent);
    }

    all_entries
}

fn get_first_entry_in_path(
    item: &str,
    span: Span,
    cwd: impl AsRef<Path>,
    paths: impl AsRef<OsStr>,
) -> Option<Value> {
    which::which_in(item, Some(paths), cwd)
        .map(|path| entry(item, path.to_string_lossy(), CommandType::External, span))
        .ok()
}

fn get_all_entries_in_path(
    item: &str,
    span: Span,
    cwd: impl AsRef<Path>,
    paths: impl AsRef<OsStr>,
) -> Vec<Value> {
    which::which_in_all(&item, Some(paths), cwd)
        .map(|iter| {
            iter.map(|path| entry(item, path.to_string_lossy(), CommandType::External, span))
                .collect()
        })
        .unwrap_or_default()
}

#[derive(Debug)]
struct WhichArgs {
    applications: Vec<Spanned<String>>,
    all: bool,
}

fn which_single(
    application: Spanned<String>,
    all: bool,
    engine_state: &EngineState,
    cwd: impl AsRef<Path>,
    paths: impl AsRef<OsStr>,
) -> Vec<Value> {
    let (external, prog_name) = if application.item.starts_with('^') {
        (true, application.item[1..].to_string())
    } else {
        (false, application.item.clone())
    };

    //If prog_name is an external command, don't search for nu-specific programs
    //If all is false, we can save some time by only searching for the first matching
    //program
    //This match handles all different cases
    match (all, external) {
        (true, true) => get_all_entries_in_path(&prog_name, application.span, cwd, paths),
        (true, false) => {
            let mut output: Vec<Value> = vec![];
            output.extend(get_entries_in_nu(
                engine_state,
                &prog_name,
                application.span,
                false,
            ));
            output.extend(get_all_entries_in_path(
                &prog_name,
                application.span,
                cwd,
                paths,
            ));
            output
        }
        (false, true) => {
            if let Some(entry) = get_first_entry_in_path(&prog_name, application.span, cwd, paths) {
                return vec![entry];
            }
            vec![]
        }
        (false, false) => {
            let nu_entries = get_entries_in_nu(engine_state, &prog_name, application.span, true);
            if !nu_entries.is_empty() {
                return vec![nu_entries[0].clone()];
            } else if let Some(entry) =
                get_first_entry_in_path(&prog_name, application.span, cwd, paths)
            {
                return vec![entry];
            }
            vec![]
        }
    }
}

fn which(
    engine_state: &EngineState,
    stack: &mut Stack,
    call: &Call,
) -> Result<PipelineData, ShellError> {
    let head = call.head;
    let which_args = WhichArgs {
        applications: call.rest(engine_state, stack, 0)?,
        all: call.has_flag(engine_state, stack, "all")?,
    };

    let mut output = vec![];

    #[allow(deprecated)]
    let cwd = env::current_dir_str(engine_state, stack)?;
    let paths = match env::path_str(engine_state, stack, head) {
        Ok(paths) => Some(paths),
        Err(ShellError::EnvVarNotFoundAtRuntime { .. }) if which_args.applications.is_empty() => {
            None
        }
        Err(error) => return Err(error),
    };

    if which_args.applications.is_empty() {
        let mut seen = std::collections::HashSet::new();
        for (name, id) in engine_state.get_decls_sorted(false) {
            let name = String::from_utf8_lossy(&name).into_owned();
            if which_args.all || seen.insert(name.clone()) {
                output.push(entry(
                    name,
                    "",
                    engine_state.get_decl(id).command_type(),
                    head,
                ));
            }
        }
        for directory in paths.iter().flat_map(std::env::split_paths) {
            let directory = if directory.is_absolute() {
                directory
            } else {
                Path::new(&cwd).join(directory)
            };
            let Ok(entries) = std::fs::read_dir(&directory) else {
                continue;
            };
            let mut entries = entries.filter_map(Result::ok).collect::<Vec<_>>();
            entries.sort_by_key(|e| e.file_name());
            for file in entries {
                let name = file.file_name().to_string_lossy().into_owned();
                if let Ok(path) = which::which_in(&name, Some(directory.as_os_str()), &cwd) {
                    if which_args.all || seen.insert(name.clone()) {
                        output.push(entry(
                            name,
                            path.to_string_lossy(),
                            CommandType::External,
                            head,
                        ));
                    }
                }
            }
        }
    }

    for app in which_args.applications {
        let values = which_single(
            app,
            which_args.all,
            engine_state,
            cwd.clone(),
            paths.clone().unwrap_or_default(),
        );
        output.extend(values);
    }

    Ok(output
        .into_iter()
        .into_pipeline_data(head, engine_state.signals().clone()))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_examples() {
        crate::test_examples(Which)
    }
}
