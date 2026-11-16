use nu_test_support::{nu, playground::Playground};

#[test]
fn null_spreads_contribute_no_items() {
    for code in [
        "[1 ...(null) 2] | to nuon",
        "const xs = [1 ...(null) 2]; $xs | to nuon",
        "def f [...xs] { $xs }; f 1 ...(null) 2 | to nuon",
        "echo 1 ...(null) 2 | to nuon",
    ] {
        let out = nu!(code);
        assert_eq!(out.out, "[1, 2]", "{code}: {}", out.err);
    }
    for code in [
        "{a: 1 ...(null)} | to nuon",
        "const r = {a: 1 ...(null)}; $r | to nuon",
    ] {
        let out = nu!(code);
        assert_eq!(out.out, "{a: 1}", "{}", out.err);
    }
    let out = nu!("nu --testbin cococo a ...(null) b");
    assert_eq!(out.out, "a b", "{}", out.err);
}

#[test]
fn dynamic_table_columns_require_strings() {
    let out = nu!("let name = 'column'; [[$name]; [42]] | get column.0");
    assert_eq!(out.out, "42", "{}", out.err);
    let out = nu!("let name: any = 42; [[$name]; [1]]");
    assert!(out.err.contains("can't convert"), "{}", out.err);
    let out = nu!("[[42]; [1]]");
    assert!(out.err.contains("must be a string"), "{}", out.err);
    let out = nu!("let name: any = 'ok'; [[$name fixed]; [1 2] [3 4]] | get fixed | to nuon");
    assert_eq!(out.out, "[2, 4]", "{}", out.err);
}

#[test]
fn streaming_table_settings_validate_and_round_trip() {
    let out =
        nu!("[$env.config.table.batch_duration $env.config.table.stream_page_size] | to nuon");
    assert_eq!(out.out, "[1000000000ns, 1000]", "{}", out.err);
    let out = nu!(
        "$env.config.table.batch_duration = 20ms; $env.config.table.stream_page_size = 2; [$env.config.table.batch_duration $env.config.table.stream_page_size] | to nuon"
    );
    assert_eq!(out.out, "[20000000ns, 2]", "{}", out.err);
    for assignment in [
        "stream_page_size = 0",
        "stream_page_size = -1",
        "stream_page_size = 65536",
        "batch_duration = -1ns",
        "stream_page_size = 'bad'",
        "batch_duration = 1",
    ] {
        let out = nu!(format!("$env.config.table.{assignment}"));
        assert!(!out.err.is_empty(), "{assignment}");
    }
}

#[test]
fn cell_paths_convert_to_normalized_strings() {
    for (input, expected) in [
        ("$.name", "$.name"),
        ("$.0.name?", "$.0.name?"),
        ("$.'two words'", "$.\"two words\""),
    ] {
        let out = nu!(format!("{input} | into string"));
        assert_eq!(out.out, expected, "{}", out.err);
    }
}

#[test]
fn block_and_subexpression_redirection_is_finished_once() {
    Playground::setup("foundation_a_redirect", |dirs, _| {
        let out = nu!(cwd: dirs.test(), "def f [] { if true { 'contents' } }; f o> block.txt; (if true { 'contents' }) o> sub.txt; [(open --raw block.txt) (open --raw sub.txt)] | to nuon");
        assert_eq!(out.out, "[contents, contents]", "{}", out.err);
    });
}

#[cfg(feature = "sqlite")]
#[test]
fn sqlite_json_columns_round_trip_structured_values() {
    let out = nu!(r#"
        stor create -t docs -c {j: JsOn, b: JSONB, text: str};
        {j: {nested: [1 null true]}, b: [[name]; [value]], text: '{"plain":1}'} | stor insert -t docs;
        stor open | query db 'select * from docs' | first | to nuon
    "#);
    assert_eq!(
        out.out, r#"{j: {nested: [1, null, true]}, b: [[name]; [value]], text: "{\"plain\":1}"}"#,
        "{}",
        out.err
    );
}

#[test]
fn upgraded_uutils_preserve_filesystem_operations() {
    Playground::setup("foundation_a_uutils", |dirs, _| {
        let out = nu!(cwd: dirs.test(), "mkdir a/b; 'value' | save a/b/file; cp a/b/file a/copy; mv a/copy a/moved; touch a/touched; [(open a/moved) ('a/touched' | path exists)] | to nuon");
        assert_eq!(out.out, "[value, true]", "{}", out.err);
    });
}

#[test]
fn example_results_have_valid_spans() {
    let out = nu!(
        "let result = (scope commands | where name == 'into string' | first | get examples | where result != null | first | get result); let s = (metadata $result).span; $s.end > $s.start"
    );
    assert_eq!(out.out, "true", "{}", out.err);
}

#[test]
fn http_timeout_category_covers_headers_and_body() {
    use std::{
        io::{Read, Write},
        net::TcpListener,
        thread,
        time::Duration,
    };
    for send_headers in [false, true] {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        listener.set_nonblocking(true).unwrap();
        let server = thread::spawn(move || {
            let deadline = std::time::Instant::now() + Duration::from_secs(5);
            let mut connection = loop {
                if let Ok((connection, _)) = listener.accept() {
                    break connection;
                }
                assert!(
                    std::time::Instant::now() < deadline,
                    "No HTTP connection received"
                );
                thread::sleep(Duration::from_millis(10));
            };
            connection
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut buf = [0; 4096];
            connection.read(&mut buf).unwrap();
            if send_headers {
                connection
                    .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 4\r\nConnection: close\r\n\r\n")
                    .unwrap();
            }
            thread::sleep(Duration::from_millis(300));
        });
        let out = nu!(format!(
            "http get --raw --max-time 50ms http://{address} | collect"
        ));
        eprintln!("timeout result: {}", out.err);
        server.join().unwrap();
        assert!(
            out.err.contains("nu::shell::io::timed_out"),
            "headers={send_headers}: {}",
            out.err
        );
        assert!(out.err.contains("Timed out"), "{}", out.err);
    }
}

#[cfg(feature = "sqlite")]
#[test]
fn sqlite_declared_jsonb_decodes_sqlite_binary_format() {
    let out = nu!(r#"
        stor create -t docs -c {j: jsonb};
        stor open | query db "insert into docs values (jsonb('{\"nested\":[1,true,null]}'))" | ignore;
        stor open | query db 'select * from docs' | get j.0.nested | to nuon
    "#);
    assert_eq!(out.out, "[1, true, null]", "{}", out.err);
}

#[test]
fn uutils_diagnostics_keep_translations_across_commands() {
    Playground::setup("foundation_a_localization", |dirs, _| {
        let out = nu!(cwd: dirs.test(), "mkdir a; 'value' | save a/file; cp a/file a/file; mv missing target");
        assert!(out.err.contains("same file"), "{}", out.err);
        assert!(!out.err.contains("cp-error-"), "{}", out.err);
        assert!(!out.err.contains("mv-error-"), "{}", out.err);
    });
}

#[test]
fn nested_example_results_have_valid_spans() {
    let out = nu!(
        "let result = (scope commands | where name == 'each' | first | get examples | where result != null | first | get result); let s = (metadata $result.0).span; $s.end > $s.start"
    );
    assert_eq!(out.out, "true", "{}", out.err);
}

#[test]
fn branch_expression_redirections_preserve_output() {
    Playground::setup("foundation_a_branch_redirect", |dirs, _| {
        for (index, expr) in [
            "if false { 'wrong' } else 'contents'",
            "if false { 'wrong' } else if true { 'contents' }",
            "match 1 { 1 => 'contents', _ => 'wrong' }",
            "match 1 { 1 => ('contents'), _ => 'wrong' }",
            "match 1 { 1 => { 'contents' }, _ => 'wrong' }",
            "match 1 { 1 => (if true { 'contents' }), _ => 'wrong' }",
            "if true { 'contents' } else 'wrong'",
            "if false { 'wrong' } else ({x: 'contents'}).x",
            "match 1 { 1 => ({x: 'contents'}).x }",
            "if false { 'wrong' } else (({x: {y: 'contents'}}).x).y",
            "match 1 { 1 => (({x: {y: 'contents'}}).x).y }",
        ]
        .iter()
        .enumerate()
        {
            let out = nu!(cwd: dirs.test(), format!("{expr} o> result{index}.txt; open --raw result{index}.txt"));
            assert!(out.status.success(), "{expr}: {}", out.err);
            assert_eq!(out.out, "contents", "{expr}: {}", out.err);
        }
        for expr in ["if false { 'wrong' }", "match 1 { 2 => 'wrong' }"] {
            let out = nu!(cwd: dirs.test(), format!("{expr} o> empty.txt; open --raw empty.txt | str length"));
            assert!(out.status.success(), "{expr}: {}", out.err);
            assert_eq!(out.out, "0", "{expr}: {}", out.err);
        }
    });
}

#[test]
fn streaming_table_public_field_types() {
    use std::{num::NonZeroU16, time::Duration};
    let config = nu_protocol::TableConfig::default();
    let duration: Duration = config.batch_duration;
    let page_size: NonZeroU16 = config.stream_page_size;
    assert_eq!(duration, Duration::from_secs(1));
    assert_eq!(page_size, NonZeroU16::new(1000).unwrap());
    let out = nu!(
        "$env.config.table.stream_page_size = 65535; $env.config.table.batch_duration = 0ns; [$env.config.table.stream_page_size $env.config.table.batch_duration] | to nuon"
    );
    assert!(out.status.success(), "{}", out.err);
    assert_eq!(out.out, "[65535, 0ns]");
}
