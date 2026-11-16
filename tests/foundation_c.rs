use nu_test_support::nu;

#[test]
fn duplicate_columns_are_catchable() {
    for flags in ["", "--guess"] {
        let result = nu!(format!(
            r#"try {{ "a a\n1 2" | detect columns {flags} | collect }} catch {{ 'caught' }}"#
        ));
        assert_eq!(result.out, "caught", "{}", result.err);
    }
    // Multiple cells overlapping a single header used to push the same key twice.
    let result = nu!(r#"try { "longcolumn x\na b        c" | detect columns } catch { 'caught' }"#);
    assert_eq!(result.out, "caught", "{}", result.err);
}

#[test]
fn namespace_help_and_parent_registration() {
    for parent in ["attr", "error", "detect", "query"] {
        let result = nu!(parent);
        assert!(result.err.is_empty(), "{}", result.err);
        assert!(result.out.contains("subcommand"), "{}", result.out);
    }
    let result = nu!(
        "let commands = scope commands | where type in [built-in keyword] | get name; $commands | where { |name| ($name | str contains ' ') and (($name | split row ' ' | drop | str join ' ') not-in $commands) } | to nuon"
    );
    assert_eq!(result.out, "[]", "{}", result.err);
}

#[test]
fn which_lists_visible_commands() {
    let result = nu!(
        r#"def foundation_custom [] {}; alias foundation_alias = echo hi; which | where command in [foundation_custom foundation_alias ls] | get command | sort | to nuon"#
    );
    assert_eq!(
        result.out, "[foundation_alias, foundation_custom, ls]",
        "{}",
        result.err
    );
}

#[cfg(unix)]
#[test]
fn which_lists_and_deduplicates_path_executables() {
    use nu_test_support::playground::Playground;
    use std::os::unix::fs::PermissionsExt;
    Playground::setup("which_foundation_paths", |dirs, sandbox| {
        sandbox.mkdir("one").mkdir("two");
        for directory in ["one", "two"] {
            let file = dirs.test().join(directory).join("foundation_executable");
            std::fs::write(&file, "#!/bin/sh\n").unwrap();
            std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        let result = nu!(cwd: dirs.test(), r#"$env.PATH = [($env.PWD | path join one) ($env.PWD | path join two)]; let first = which | where command == foundation_executable; let all = which -a | where command == foundation_executable; [($first | length) ($all | length) ($first.0.path | str ends-with 'one/foundation_executable')] | to nuon"#);
        assert_eq!(result.out, "[1, 2, true]", "{}", result.err);
    });
}

#[test]
fn powers_associate_right_and_other_operators_keep_precedence() {
    for (expression, expected) in [
        ("2 ** 1 ** 2", "2"),
        ("2 ** 3 ** 2", "512"),
        ("2 * 2 ** 3 ** 2 + 1", "1025"),
        ("20 - 5 - 2", "13"),
        ("(2 ** 3) ** 2", "64"),
    ] {
        let result = nu!(expression);
        assert_eq!(result.out, expected, "{expression}: {}", result.err);
    }
}

#[test]
fn bare_interpolation_at_both_ends() {
    let result = nu!("echo (100 + 20 + 3)/bar/(300 + 20 + 1)");
    assert_eq!(result.out, "123/bar/321", "{}", result.err);
}

#[test]
fn nuon_equals_are_quoted_and_round_trip() {
    let result = nu!(r#"[{'=': '=' 'a=': '=a'} {'=': 'a=' 'a=': '='}] | to nuon"#);
    assert!(result.out.contains(r#""=""#), "{}", result.out);
    assert!(result.out.contains(r#""a=""#), "{}", result.out);
    assert!(result.out.contains(r#""=a""#), "{}", result.out);
    let result = nu!(r#"let data = {'=': '=' 'a=': '=a'}; ($data | to nuon | from nuon) == $data"#);
    assert_eq!(result.out, "true", "{}", result.err);
}

#[test]
fn missing_script_points_to_commandline() {
    let result = nu!("nu non-existent-script.nu foo bar");
    assert!(!result.err.contains(".rs"), "{}", result.err);
    assert!(result.err.contains("commandline"), "{}", result.err);
    assert!(
        result.err.contains("non-existent-script.nu"),
        "{}",
        result.err
    );
}

#[test]
fn binary_endian_numeric_and_compact() {
    for (input, little, big) in [
        ("258", "0x[0201000000000000]", "0x[0000000000000102]"),
        ("true", "0x[0100000000000000]", "0x[0000000000000001]"),
        ("258ns", "0x[0201000000000000]", "0x[0000000000000102]"),
        ("258b", "0x[0201000000000000]", "0x[0000000000000102]"),
        ("1.0", "0x[000000000000f03f]", "0x[3ff0000000000000]"),
    ] {
        for (endian, expected) in [("little", little), ("big", big)] {
            let result = nu!(format!("({input} | into binary -e {endian}) == {expected}"));
            assert_eq!(result.out, "true", "{input}/{endian}: {}", result.err);
        }
    }
    for (endian, expected) in [("little", "0x[0201]"), ("big", "0x[0102]")] {
        let result = nu!(format!(
            "(258 | into binary -e {endian} --compact) == {expected}"
        ));
        assert_eq!(result.out, "true", "{}", result.err);
    }
    let result = nu!("1 | into binary -e sideways");
    assert!(result.err.contains("type_mismatch"), "{}", result.err);
    let result = nu!("(258 | into binary -e native) == (258 | into binary)");
    assert_eq!(result.out, "true", "{}", result.err);
}

#[test]
fn markdown_spacing_escaping_and_groups() {
    for (script, expected) in [
        (
            "[[foo bar]; [1 2]] | to md",
            "| foo | bar |\n| --- | --- |\n| 1 | 2 |",
        ),
        (
            "[[foo bar]; [1 2]] | to md --center [bar]",
            "| foo | bar |\n| --- |:---:|\n| 1 | 2 |",
        ),
        (
            r#"[{"": "" foo: "a|b"}] | to md"#,
            "|  | foo |\n| --- | --- |\n|  | a\\|b |",
        ),
        (
            r#"[{a: "*x* <b>"}] | to md -m"#,
            "| a |\n| --- |\n| \\*x\\* \\<b\\> |",
        ),
        (
            r#"[{a: '<b/>&"'}] | to md -t"#,
            "| a |\n| --- |\n| &lt;b&#x2f;&gt;&amp;&quot; |",
        ),
        (
            r#"[{a: '*<b>*'}] | to md -a"#,
            "| a |\n| --- |\n| \\*&lt;b&gt;\\* |",
        ),
        (
            "[{foo: 1} {bar: 2}] | to md --per-element",
            "| foo |\n| --- |\n| 1 |\n\n| bar |\n| --- |\n| 2 |",
        ),
    ] {
        let result = nu!(script);
        // nu! strips newlines from stdout.
        assert_eq!(
            result.out,
            expected.replace('\n', ""),
            "{script}: {}",
            result.err
        );
    }
}

#[test]
fn markdown_groups_retain_blank_lines() {
    let result = nu!(r#"([{foo: 1} {bar: 2}] | to md --per-element) | str contains "\n\n""#);
    assert_eq!(result.out, "true", "{}", result.err);
}

#[test]
fn which_without_path_still_lists_internal_commands() {
    let result = nu!("hide-env PATH; which | where command == ls | get type.0");
    assert_eq!(result.out, "built-in", "{}", result.err);
}

#[test]
fn binary_cell_paths_and_non_numeric_values() {
    let result = nu!("([{n: 258}] | into binary -e big n | get 0.n) == 0x[0000000000000102]");
    assert_eq!(result.out, "true", "{}", result.err);
    for value in ["'abc'", "0x[010200]", "2025-01-01"] {
        let result = nu!(format!(
            "({value} | into binary -e little) == ({value} | into binary -e big)"
        ));
        assert_eq!(result.out, "true", "{value}: {}", result.err);
    }
}
