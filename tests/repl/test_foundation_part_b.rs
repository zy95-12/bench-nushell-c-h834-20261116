use crate::repl::tests::{TestResult, fail_test, run_test};

#[test]
fn negated_string_operators() -> TestResult {
    for (source, expected) in [
        (r#""hello" not-starts-with "he""#, "false"),
        (r#""hello" not-starts-with "HE""#, "true"),
        (r#""hello" not-ends-with "lo""#, "false"),
        (r#""hello" not-ends-with "LO""#, "true"),
        (r#""" not-starts-with """#, "false"),
        (
            r#"const result = ("abc" not-starts-with "z"); $result"#,
            "true",
        ),
        (
            r#"["abc" "def"] | where $it not-starts-with "a" | first"#,
            "def",
        ),
        (
            r#"["abc" "def"] | where $it not-ends-with "f" | first"#,
            "abc",
        ),
        (
            r#"help operators | where operator in [not-starts-with not-ends-with] | length"#,
            "2",
        ),
    ] {
        run_test(source, expected)?;
    }
    fail_test(r#"1 not-starts-with "a""#, "unsupported")
}

#[test]
fn loop_control_is_checked_before_execution() -> TestResult {
    for statement in ["break", "continue"] {
        for source in [
            statement.to_string(),
            format!("do {{ {statement} }}"),
            format!("if false {{ {statement} }}"),
            format!("def bad [] {{ {statement} }}"),
            format!("null | export-env {{ {statement} }}"),
        ] {
            fail_test(&source, "nu::compile::not_in_a_loop")?;
        }
    }
    run_test(
        "module test { export-env { $env.PART_B = 42 } }; use test; $env.PART_B",
        "42",
    )?;
    fail_test(
        "module test { export-env { break } }",
        "nu::compile::not_in_a_loop",
    )?;
    run_test("loop { break }; 42", "42")?;
    run_test("mut x = 0; while $x < 3 { $x += 1; continue }; $x", "3")?;
    run_test(
        "mut x = 0; for i in 1..3 { if $i == 2 { continue }; $x += $i }; $x",
        "4",
    )
}

#[test]
fn row_conditions_require_bool() -> TestResult {
    fail_test("[1 2] | where 42", "expected bool")?;
    fail_test(r#"[1 2] | where ("yes")"#, "expected bool")?;
    run_test("[1 2] | where $it > 1 | first", "2")
}

#[test]
fn row_conditions_preserve_quoted_boolean_columns() -> TestResult {
    run_test(r#"[["a b"]; [true] [false]] | where "a b" | length"#, "1")?;
    run_test(r#"[[flag]; [true] [false]] | where "flag" | length"#, "1")
}

#[test]
fn unicode_scalar_lengths() -> TestResult {
    for (source, expected) in [
        ("'hällo' | str length --chars", "5"),
        ("'hello' | str length -c", "5"),
        ("'hello' | str length", "5"),
        ("'hällo' | str length", "6"),
        ("'é' | str length -c", "2"),
        ("'é' | str length -g", "1"),
        ("const n = ('hällo' | str length -c); $n", "5"),
    ] {
        run_test(source, expected)?;
    }
    fail_test("'hello' | str length -c -g", "incompatible_parameters")?;
    fail_test("'hello' | str length -c -b", "incompatible_parameters")?;
    fail_test(
        "const n = ('hello' | str length -c -g)",
        "incompatible_parameters",
    )
}

#[test]
fn relative_paths_preserve_platform_case_rules() -> TestResult {
    run_test("'foo/bar/baz' | path relative-to 'foo/bar'", "baz")?;
    fail_test("'foo/bar' | path relative-to 'other'", "convert")?;
    if cfg!(any(target_os = "windows", target_os = "macos")) {
        run_test("'foo/Bar/baz' | path relative-to 'FOO/bar'", "baz")
    } else {
        fail_test("'foo/Bar/baz' | path relative-to 'FOO/bar'", "convert")
    }
}
