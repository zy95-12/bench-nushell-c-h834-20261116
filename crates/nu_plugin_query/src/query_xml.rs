use crate::Query;
use nu_plugin::{EngineInterface, EvaluatedCall, SimplePluginCommand};
use nu_protocol::{Category, LabeledError, Record, Signature, Span, Spanned, SyntaxShape, Value};
use sxd_document::parser;
use sxd_xpath::{Context, Factory};

pub struct QueryXml;

impl SimplePluginCommand for QueryXml {
    type Plugin = Query;

    fn name(&self) -> &str {
        "query xml"
    }

    fn description(&self) -> &str {
        "execute xpath query on xml"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .required("query", SyntaxShape::String, "xpath query")
            .named(
                "namespaces",
                SyntaxShape::Record(vec![]),
                "map of prefixes to namespace URIs",
                Some('n'),
            )
            .switch("output-string-value", "include node string values", None)
            .switch("output-type", "include node types", None)
            .switch("output-names", "include node names and namespaces", None)
            .category(Category::Filters)
    }

    fn run(
        &self,
        _plugin: &Query,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        input: &Value,
    ) -> Result<Value, LabeledError> {
        let query: Option<Spanned<String>> = call.opt(0)?;
        let namespaces: Option<Record> = call.get_flag::<Record>("namespaces")?;

        execute_xpath_query(call, input, query, namespaces)
    }
}

pub fn execute_xpath_query(
    call: &EvaluatedCall,
    input: &Value,
    query: Option<Spanned<String>>,
    namespaces: Option<Record>,
) -> Result<Value, LabeledError> {
    let (query_string, span) = match &query {
        Some(v) => (&v.item, v.span),
        None => {
            return Err(
                LabeledError::new("problem with input data").with_label("query missing", call.head)
            );
        }
    };

    let xpath = build_xpath(query_string, span)?;
    let input_string = input.coerce_str()?;
    let package = parser::parse(&input_string);

    if let Err(err) = package {
        return Err(
            LabeledError::new("Invalid XML document").with_label(err.to_string(), input.span())
        );
    }

    let package = package.expect("invalid xml document");

    let document = package.as_document();
    let mut context = Context::new();
    context.set_namespace("xml", "http://www.w3.org/XML/1998/namespace");

    for (prefix, uri) in namespaces.unwrap_or_default().into_iter() {
        context.set_namespace(prefix.as_str(), uri.into_string()?.as_str());
    }

    // leaving this here for augmentation at some point
    // build_variables(&arguments, &mut context);
    // build_namespaces(&arguments, &mut context);
    let res = xpath.evaluate(&context, document.root());

    let output_type = call.has_flag("output-type")?;
    let output_names = call.has_flag("output-names")?;
    let output_string_value =
        call.has_flag("output-string-value")? || !(output_type || output_names);

    match res.map_err(|err| {
        LabeledError::new("xpath query error").with_label(err.to_string(), call.head)
    })? {
        sxd_xpath::Value::Boolean(b) => Ok(Value::bool(b, call.head)),
        sxd_xpath::Value::Number(n) => Ok(Value::float(n, call.head)),
        sxd_xpath::Value::String(s) => Ok(Value::string(s, call.head)),
        sxd_xpath::Value::Nodeset(ns) => {
            let records = ns
                .document_order()
                .into_iter()
                .map(|node| {
                    let mut record = Record::new();
                    if output_string_value {
                        record.push(
                            "string_value",
                            Value::string(node.string_value(), call.head),
                        );
                    }
                    if output_type {
                        use sxd_xpath::nodeset::Node;
                        let kind = match node {
                            Node::Element(_) => "element",
                            Node::Attribute(_) => "attribute",
                            Node::Text(_) => "text",
                            Node::Comment(_) => "comment",
                            Node::ProcessingInstruction(_) => "processing_instruction",
                            Node::Root(_) => "root",
                            Node::Namespace(_) => "namespace",
                        };
                        record.push("type", Value::string(kind, call.head));
                    }
                    if output_names {
                        let name = node.expanded_name();
                        let optional_string = |s: Option<&str>| match s {
                            Some(s) => Value::string(s, call.head),
                            None => Value::nothing(call.head),
                        };
                        record.push("local_name", optional_string(name.map(|n| n.local_part())));
                        record.push(
                            "namespace",
                            optional_string(name.and_then(|n| n.namespace_uri())),
                        );
                        record.push(
                            "prefixed_name",
                            optional_string(node.prefixed_name().as_deref()),
                        );
                    }
                    Value::record(record, call.head)
                })
                .collect();
            Ok(Value::list(records, call.head))
        }
    }
}

fn build_xpath(xpath_str: &str, span: Span) -> Result<sxd_xpath::XPath, LabeledError> {
    let factory = Factory::new();

    match factory.build(xpath_str) {
        Ok(xpath) => xpath.ok_or_else(|| {
            LabeledError::new("invalid xpath query").with_label("the query must not be empty", span)
        }),
        Err(err) => Err(LabeledError::new("invalid xpath query").with_label(err.to_string(), span)),
    }
}

#[cfg(test)]
mod tests {
    use super::execute_xpath_query as query;
    use nu_plugin::EvaluatedCall;
    use nu_protocol::{Span, Spanned, Value, record};

    #[test]
    fn position_function_in_predicate() {
        let call = EvaluatedCall {
            head: Span::test_data(),
            positional: vec![],
            named: vec![],
        };

        let text = Value::string(
            r#"<?xml version="1.0" encoding="UTF-8"?><a><b/><b/></a>"#,
            Span::test_data(),
        );

        let spanned_str: Spanned<String> = Spanned {
            item: "count(//a/*[position() = 2])".to_string(),
            span: Span::test_data(),
        };

        let actual = query(&call, &text, Some(spanned_str), None).expect("test should not fail");

        let expected = Value::test_float(1.0);

        assert_eq!(actual, expected);
    }

    #[test]
    fn functions_implicitly_coerce_argument_types() {
        let call = EvaluatedCall {
            head: Span::test_data(),
            positional: vec![],
            named: vec![],
        };

        let text = Value::string(
            r#"<?xml version="1.0" encoding="UTF-8"?><a>true</a>"#,
            Span::test_data(),
        );

        let spanned_str: Spanned<String> = Spanned {
            item: "count(//*[contains(., true)])".to_string(),
            span: Span::test_data(),
        };

        let actual = query(&call, &text, Some(spanned_str), None).expect("test should not fail");

        let expected = Value::test_float(1.0);

        assert_eq!(actual, expected);
    }

    #[test]
    fn namespaces_are_used() {
        let call = EvaluatedCall {
            head: Span::test_data(),
            positional: vec![],
            named: vec![],
        };

        // document uses `dp` ("document prefix") as a prefix
        let text = Value::string(
            r#"<?xml version="1.0" encoding="UTF-8"?><a xmlns:dp="http://example.com/ns"><dp:b>yay</dp:b></a>"#,
            Span::test_data(),
        );

        // but query uses `qp` ("query prefix") as a prefix
        let namespaces = record! {
            "qp" => Value::string("http://example.com/ns", Span::test_data()),
        };

        let spanned_str: Spanned<String> = Spanned {
            item: "//qp:b/text()".to_string(),
            span: Span::test_data(),
        };

        let actual =
            query(&call, &text, Some(spanned_str), Some(namespaces)).expect("test should not fail");

        let expected = Value::list(
            vec![Value::test_record(record! {
                "string_value" => Value::string("yay", Span::test_data()),
            })],
            Span::test_data(),
        );

        // and yet it should work regardless
        assert_eq!(actual, expected);
    }
    fn evaluate(
        xml: &str,
        xpath: &str,
        flags: &[&str],
        namespaces: Option<nu_protocol::Record>,
    ) -> Value {
        let call = EvaluatedCall {
            head: Span::test_data(),
            positional: vec![],
            named: flags
                .iter()
                .map(|name| {
                    (
                        Spanned {
                            item: name.to_string(),
                            span: Span::test_data(),
                        },
                        None,
                    )
                })
                .collect(),
        };
        query(
            &call,
            &Value::test_string(xml),
            Some(Spanned {
                item: xpath.into(),
                span: Span::test_data(),
            }),
            namespaces,
        )
        .expect("valid query")
    }

    #[test]
    fn scalar_results_are_native_even_with_output_flags() {
        for flags in [vec![], vec!["output-type", "output-names"]] {
            for (xpath, expected) in [
                ("true()", Value::test_bool(true)),
                ("false()", Value::test_bool(false)),
                ("1.5", Value::test_float(1.5)),
                ("string(/a)", Value::test_string("hello")),
            ] {
                assert_eq!(evaluate("<a>hello</a>", xpath, &flags, None), expected);
            }
        }
    }

    #[test]
    fn every_output_flag_combination_has_selected_columns_in_fixed_order() {
        let flags = ["output-string-value", "output-type", "output-names"];
        for mask in 0..8 {
            // Supply flags in reverse order to ensure column order is independent.
            let selected: Vec<_> = (0..3)
                .rev()
                .filter(|i| mask & (1 << i) != 0)
                .map(|i| flags[i])
                .collect();
            let actual = evaluate("<a>hello</a>", "/a", &selected, None);
            let mut expected = nu_protocol::Record::new();
            if mask == 0 || mask & 1 != 0 {
                expected.push("string_value", Value::test_string("hello"));
            }
            if mask & 2 != 0 {
                expected.push("type", Value::test_string("element"));
            }
            if mask & 4 != 0 {
                expected.push("local_name", Value::test_string("a"));
                expected.push("namespace", Value::nothing(Span::test_data()));
                expected.push("prefixed_name", Value::test_string("a"));
            }
            let rows = actual.as_list().unwrap();
            assert_eq!(rows.len(), 1);
            let row = rows[0].as_record().unwrap();
            assert_eq!(
                row.columns().collect::<Vec<_>>(),
                expected.columns().collect::<Vec<_>>()
            );
            assert_eq!(actual, Value::test_list(vec![Value::test_record(expected)]));
        }
    }

    #[test]
    fn node_types_and_names_follow_xpath_node_kinds() {
        let xml = r#"<a xmlns:p="urn:p" p:attr="v"><p:b/>text<!--note--><?target data?></a>"#;
        for (xpath, kind, local, namespace, prefixed) in [
            ("/", "root", None, None, None),
            ("/a/p:b", "element", Some("b"), Some("urn:p"), Some("p:b")),
            (
                "/a/@p:attr",
                "attribute",
                Some("attr"),
                Some("urn:p"),
                Some("p:attr"),
            ),
            ("/a/text()", "text", None, None, None),
            ("/a/comment()", "comment", None, None, None),
            (
                "/a/processing-instruction()",
                "processing_instruction",
                Some("target"),
                None,
                Some("target"),
            ),
            ("/a/namespace::p", "namespace", Some("p"), None, Some("p")),
        ] {
            let actual = evaluate(
                xml,
                xpath,
                &["output-type", "output-names"],
                Some(record! {
                    "p" => Value::test_string("urn:p"),
                }),
            );
            let value = |s: Option<&str>| {
                s.map(Value::test_string)
                    .unwrap_or_else(|| Value::nothing(Span::test_data()))
            };
            assert_eq!(
                actual,
                Value::test_list(vec![Value::test_record(record! {
                    "type" => Value::test_string(kind),
                    "local_name" => value(local),
                    "namespace" => value(namespace),
                    "prefixed_name" => value(prefixed),
                })]),
                "{xpath}"
            );
        }
    }

    #[test]
    fn xml_prefix_is_available_by_default_and_can_be_overridden() {
        let xml = r#"<a xml:lang="en" xmlns:p="urn:custom" p:lang="custom"/>"#;
        assert_eq!(
            evaluate(xml, "string(/a/@xml:lang)", &[], None),
            Value::test_string("en")
        );
        assert_eq!(
            evaluate(
                xml,
                "string(/a/@xml:lang)",
                &[],
                Some(record! {
                    "xml" => Value::test_string("urn:custom"),
                })
            ),
            Value::test_string("custom")
        );
    }

    #[test]
    fn nodesets_preserve_document_order_and_empty_results() {
        assert_eq!(
            evaluate("<a><b>first</b><b>second</b></a>", "/a/b", &[], None),
            Value::test_list(
                vec!["first", "second"]
                    .into_iter()
                    .map(|text| Value::test_record(record! {
                        "string_value" => Value::test_string(text),
                    }))
                    .collect()
            )
        );
        assert_eq!(
            evaluate("<a/>", "/a/missing", &[], None),
            Value::test_list(vec![])
        );
    }
}
