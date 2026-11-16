use nu_test_support::nu;

#[test]
fn sets_stream_from_internal_command_as_binary() {
    let result = nu!("seq 1 10 | to text | into binary | describe");
    assert_eq!("binary (stream)", result.out);
}

#[test]
fn sets_stream_from_external_command_as_binary() {
    let result = nu!("^nu --testbin cococo | into binary | describe");
    assert_eq!("binary (stream)", result.out);
}

#[test]
fn compact_non_numeric_values_ignore_endian() {
    for value in [
        r#""\u{0}abc\u{0}""#,
        "0x[00010200]",
        "0x[0000]",
        "2025-01-01",
    ] {
        let result = nu!(format!(
            "let value = {value}; \
             let expected = ($value | into binary --compact); \
             [little big native] | all {{|endian| \
                 ($value | into binary --compact --endian $endian) == $expected \
             }}"
        ));
        assert!(result.err.is_empty(), "{value}: {}", result.err);
        assert_eq!(result.out, "true", "{value}: {}", result.err);
    }
}
