use super::PathSubcommandArguments;
use nu_engine::command_prelude::*;
use nu_path::expand_to_real_path;
use nu_protocol::engine::StateWorkingSet;
use std::path::Path;

struct Arguments {
    path: Spanned<String>,
}

impl PathSubcommandArguments for Arguments {}

#[derive(Clone)]
pub struct PathRelativeTo;

impl Command for PathRelativeTo {
    fn name(&self) -> &str {
        "path relative-to"
    }

    fn signature(&self) -> Signature {
        Signature::build("path relative-to")
            .input_output_types(vec![
                (Type::String, Type::String),
                (
                    Type::List(Box::new(Type::String)),
                    Type::List(Box::new(Type::String)),
                ),
            ])
            .required(
                "path",
                SyntaxShape::String,
                "Parent shared with the input path.",
            )
            .category(Category::Path)
    }

    fn description(&self) -> &str {
        "Express a path as relative to another path."
    }

    fn extra_description(&self) -> &str {
        r#"Can be used only when the input and the argument paths are either both
absolute or both relative. The argument path needs to be a parent of the input
path."#
    }

    fn is_const(&self) -> bool {
        true
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        let head = call.head;
        let args = Arguments {
            path: call.req(engine_state, stack, 0)?,
        };

        // This doesn't match explicit nulls
        if matches!(input, PipelineData::Empty) {
            return Err(ShellError::PipelineEmpty { dst_span: head });
        }
        input.map(
            move |value| super::operate(&relative_to, &args, value, head),
            engine_state.signals(),
        )
    }

    fn run_const(
        &self,
        working_set: &StateWorkingSet,
        call: &Call,
        input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        let head = call.head;
        let args = Arguments {
            path: call.req_const(working_set, 0)?,
        };

        // This doesn't match explicit nulls
        if matches!(input, PipelineData::Empty) {
            return Err(ShellError::PipelineEmpty { dst_span: head });
        }
        input.map(
            move |value| super::operate(&relative_to, &args, value, head),
            working_set.permanent().signals(),
        )
    }

    #[cfg(windows)]
    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                description: "Find a relative path from two absolute paths",
                example: r"'C:\Users\viking' | path relative-to 'C:\Users'",
                result: Some(Value::test_string(r"viking")),
            },
            Example {
                description: "Find a relative path from absolute paths in list",
                example: r"[ C:\Users\viking, C:\Users\spam ] | path relative-to C:\Users",
                result: Some(Value::test_list(vec![
                    Value::test_string("viking"),
                    Value::test_string("spam"),
                ])),
            },
            Example {
                description: "Find a relative path from two relative paths",
                example: r"'eggs\bacon\sausage\spam' | path relative-to 'eggs\bacon\sausage'",
                result: Some(Value::test_string(r"spam")),
            },
        ]
    }

    #[cfg(not(windows))]
    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                description: "Find a relative path from two absolute paths",
                example: r"'/home/viking' | path relative-to '/home'",
                result: Some(Value::test_string(r"viking")),
            },
            Example {
                description: "Find a relative path from absolute paths in list",
                example: r"[ /home/viking, /home/spam ] | path relative-to '/home'",
                result: Some(Value::test_list(vec![
                    Value::test_string("viking"),
                    Value::test_string("spam"),
                ])),
            },
            Example {
                description: "Find a relative path from two relative paths",
                example: r"'eggs/bacon/sausage/spam' | path relative-to 'eggs/bacon/sausage'",
                result: Some(Value::test_string(r"spam")),
            },
        ]
    }
}

fn relative_to(path: &Path, span: Span, args: &Arguments) -> Value {
    let lhs = expand_to_real_path(path);
    let rhs = expand_to_real_path(&args.path.item);
    match lhs.strip_prefix(&rhs) {
        Ok(p) => Value::string(p.to_string_lossy(), span),
        Err(e) => {
            if is_case_insensitive_filesystem() {
                if let Some(relative) = try_case_insensitive_strip_prefix(&lhs, &rhs) {
                    return Value::string(relative.to_string_lossy(), span);
                }
            }
            Value::error(
                ShellError::CantConvert {
                    to_type: e.to_string(),
                    from_type: "string".into(),
                    span,
                    help: None,
                },
                span,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_examples() {
        use crate::test_examples;

        test_examples(PathRelativeTo {})
    }
}

fn is_case_insensitive_filesystem() -> bool {
    cfg!(any(target_os = "windows", target_os = "macos"))
}

fn try_case_insensitive_strip_prefix(lhs: &Path, rhs: &Path) -> Option<std::path::PathBuf> {
    use std::path::Component;
    let mut remaining = lhs.components();
    for expected in rhs.components() {
        let actual = remaining.next()?;
        let equal = match (actual, expected) {
            (Component::Normal(a), Component::Normal(b)) => {
                a.to_string_lossy().to_lowercase() == b.to_string_lossy().to_lowercase()
            }
            (Component::Prefix(a), Component::Prefix(b)) => {
                use std::path::Prefix;
                let eq = |a: &std::ffi::OsStr, b: &std::ffi::OsStr| {
                    a.to_string_lossy().to_lowercase() == b.to_string_lossy().to_lowercase()
                };
                match (a.kind(), b.kind()) {
                    (Prefix::UNC(a_server, a_share), Prefix::UNC(b_server, b_share))
                    | (
                        Prefix::VerbatimUNC(a_server, a_share),
                        Prefix::VerbatimUNC(b_server, b_share),
                    ) => eq(a_server, b_server) && eq(a_share, b_share),
                    _ => a == b,
                }
            }
            _ => actual == expected,
        };
        if !equal {
            return None;
        }
    }
    Some(remaining.as_path().to_path_buf())
}

#[cfg(test)]
mod case_prefix_tests {
    use super::*;

    #[cfg(windows)]
    #[test]
    fn case_fallback_handles_unc_prefixes_and_preserves_boundaries() {
        for (lhs, rhs) in [
            (r"\\Server\Share\Folder\File.TXT", r"\\server\share\folder"),
            (
                r"\\?\UNC\Server\Share\Folder\File.TXT",
                r"\\?\UNC\server\share\folder",
            ),
        ] {
            assert!(Path::new(lhs).strip_prefix(rhs).is_err());
            assert_eq!(
                try_case_insensitive_strip_prefix(Path::new(lhs), Path::new(rhs)),
                Some(std::path::PathBuf::from("File.TXT"))
            );
        }
        for (lhs, rhs) in [
            (r"\\Server\Share\Folder", r"\\server\other"),
            (r"\\Server\Share\Folder", r"\\other\share"),
            (r"\\Server\Share\Folder", r"\\server\share\fold"),
            (r"\\?\UNC\Server\Share\Folder", r"\\server\share"),
            (r"\\Server\Share\Folder", r"\\?\UNC\server\share"),
            (r"C:\Folder", r"D:\folder"),
            (r"C:\Folder", r"C:folder"),
        ] {
            assert!(try_case_insensitive_strip_prefix(Path::new(lhs), Path::new(rhs)).is_none());
        }
    }

    #[test]
    fn case_fallback_compares_whole_components_and_keeps_suffix() {
        assert_eq!(
            try_case_insensitive_strip_prefix(Path::new("/Foo/BAR/Baz"), Path::new("/foo/bar")),
            Some(std::path::PathBuf::from("Baz"))
        );
        assert_eq!(
            try_case_insensitive_strip_prefix(Path::new("/FOO"), Path::new("/foo")),
            Some(std::path::PathBuf::new())
        );
        for (lhs, rhs) in [
            ("/foobar/x", "/foo"),
            ("/foo", "/foo/bar"),
            ("foo/bar", "/foo"),
        ] {
            assert!(try_case_insensitive_strip_prefix(Path::new(lhs), Path::new(rhs)).is_none());
        }
        assert_eq!(
            is_case_insensitive_filesystem(),
            cfg!(any(target_os = "windows", target_os = "macos"))
        );
    }
}
