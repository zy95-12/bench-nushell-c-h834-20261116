use nu_protocol::{Span, engine::StateWorkingSet};
use quickcheck_macros::quickcheck;

#[quickcheck]
fn quickcheck_parse(data: String) -> bool {
    let (tokens, err) = nu_parser::lex(data.as_bytes(), 0, b"", b"", true);

    if err.is_none() {
        let context = crate::create_default_context();
        {
            let mut working_set = StateWorkingSet::new(&context);
            let _ = working_set.add_file("quickcheck".into(), data.as_bytes());

            let _ =
                nu_parser::parse_block(&mut working_set, &tokens, Span::new(0, 0), false, false);
        }
    }
    true
}

#[test]
fn fr3_export_def_parameters_do_not_leak() {
    let engine = crate::create_default_context();
    let mut ws = StateWorkingSet::new(&engine);
    nu_parser::parse(
        &mut ws,
        None,
        b"export def foo [leaked: int] { $leaked }; $leaked",
        false,
    );
    assert!(
        ws.parse_errors
            .iter()
            .any(|e| matches!(e, nu_protocol::ParseError::VariableNotFound(..))),
        "{:?}",
        ws.parse_errors
    );
    assert!(ws.find_variable(b"leaked").is_none());
}

#[test]
fn fr5_redefined_parser_keywords_do_not_panic() {
    for input in [
        "def def [] { 1",
        "def def [] {}; def foo [] { 1",
        "extern extern [x",
        "extern extern []; extern foo [x",
        "def extern [] {}; extern foo [x",
        "extern def []; def foo [] { 1",
    ] {
        let engine = crate::create_default_context();
        let mut ws = StateWorkingSet::new(&engine);
        nu_parser::parse(&mut ws, None, input.as_bytes(), false);
        assert!(
            ws.parse_errors
                .iter()
                .any(|e| matches!(e, nu_protocol::ParseError::Unclosed(..))),
            "{input}: {:?}",
            ws.parse_errors
        );
    }
}

#[test]
fn fr6_bare_comma_type_is_unknown() {
    let engine = crate::create_default_context();
    let mut ws = StateWorkingSet::new(&engine);
    nu_parser::parse(&mut ws, None, b"let x: , = 1", false);
    assert!(
        ws.parse_errors
            .iter()
            .any(|e| matches!(e, nu_protocol::ParseError::UnknownType(..))),
        "{:?}",
        ws.parse_errors
    );
}

#[test]
fn fr1_external_subexpressions_evaluate_in_order() {
    use nu_protocol::{
        ast::{Expr, ExternalArgument},
        debugger::WithoutDebug,
        engine::Stack,
    };
    for (input, expected) in [
        ("^echo pre(1 + 2)mid(4)post", "pre3mid4post"),
        ("^echo pre((1 + 2))post", "pre3post"),
        ("^echo (1)(2)", "12"),
        ("^echo (1)(2)(3)", "123"),
        ("^echo pre(1)(2)post", "pre12post"),
        ("^echo '(1)(2)'", "(1)(2)"),
        ("^echo (1 + 2)post", "3post"),
        ("^echo pre('a)b')post", "prea)bpost"),
        ("^echo pre\"(1 + 2)\"post", "pre(1 + 2)post"),
    ] {
        let mut engine = crate::create_default_context();
        let mut ws = StateWorkingSet::new(&engine);
        let block = nu_parser::parse(&mut ws, None, input.as_bytes(), false);
        assert!(ws.parse_errors.is_empty(), "{input}: {:?}", ws.parse_errors);
        let Expr::ExternalCall(_, args) = &block.pipelines[0].elements[0].expr.expr else {
            panic!()
        };
        let ExternalArgument::Regular(arg) = &args[0] else {
            panic!()
        };
        let mut evaluated = block.as_ref().clone();
        evaluated.pipelines[0].elements[0].expr = arg.clone();
        evaluated.ir_block = Some(nu_engine::compile(&ws, &evaluated).unwrap());
        engine.merge_delta(ws.render()).unwrap();
        let value = nu_engine::eval_block::<WithoutDebug>(
            &engine,
            &mut Stack::new(),
            &evaluated,
            nu_protocol::PipelineData::empty(),
        )
        .unwrap()
        .into_value(Span::unknown())
        .unwrap();
        assert_eq!(value.coerce_string().unwrap(), expected);
    }
}

#[test]
fn fr5_unclosed_signature_precedes_missing_body() {
    for input in ["def def (=a|s)>", "extern extern (=a|s)>"] {
        let engine = crate::create_default_context();
        let mut ws = StateWorkingSet::new(&engine);
        nu_parser::parse(&mut ws, None, input.as_bytes(), false);
        let error = ws
            .parse_errors
            .first()
            .expect("malformed signature must fail");
        assert!(
            matches!(error, nu_protocol::ParseError::Unclosed(..)),
            "{input}: {error:?}"
        );
        assert!(error.to_string().contains("Unclosed delimiter"));
    }
}
