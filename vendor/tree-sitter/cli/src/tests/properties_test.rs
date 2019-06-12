use super::helpers::fixtures::get_language;
use crate::properties;
use serde_derive::Deserialize;
use tree_sitter::{Parser, PropertySheet};

#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
struct Properties {
    a: Option<String>,
    b: Option<String>,
}

#[test]
fn test_walk_with_properties_with_nth_child() {
    let language = get_language("javascript");
    let property_sheet = PropertySheet::<Properties>::new(
        language,
        &properties::generate_property_sheet_string(
            "/some/path.css",
            "
                binary_expression > identifier:nth-child(2) {
                    a: x;
                }

                binary_expression > identifier {
                    a: y;
                }

                identifier {
                    a: z;
                }
            ",
        )
        .unwrap(),
    )
    .unwrap();

    let source_code = "a = b || c;";

    let mut parser = Parser::new();
    parser.set_language(language).unwrap();
    let tree = parser.parse(source_code, None).unwrap();

    let mut cursor = tree.walk_with_properties(&property_sheet, source_code.as_bytes());
    assert_eq!(cursor.node().kind(), "program");
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind(), "expression_statement");
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind(), "assignment_expression");

    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind(), "identifier");
    assert_eq!(*cursor.node_properties(), Properties { a: Some("z".to_string()), b: None });

    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind(), "=");
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind(), "binary_expression");

    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind(), "identifier");
    assert_eq!(*cursor.node_properties(), Properties { a: Some("y".to_string()), b: None });

    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind(), "||");
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind(), "identifier");
    assert_eq!(*cursor.node_properties(), Properties { a: Some("x".to_string()), b: None });
}

#[test]
fn test_walk_with_properties_with_regexes() {
    let language = get_language("javascript");
    let property_sheet = PropertySheet::<Properties>::new(
        language,
        &properties::generate_property_sheet_string(
            "/some/path.css",
            "
                identifier {
                    &[text='^[A-Z]'] {
                        a: y;
                    }

                    &[text='^[A-Z_]+$'] {
                        a: z;
                    }

                    a: x;
                }
            ",
        )
        .unwrap(),
    )
    .unwrap();

    let source_code = "const ABC = Def(ghi);";

    let mut parser = Parser::new();
    parser.set_language(language).unwrap();
    let tree = parser.parse(source_code, None).unwrap();

    let mut cursor = tree.walk_with_properties(&property_sheet, source_code.as_bytes());
    assert_eq!(cursor.node().kind(), "program");
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind(), "lexical_declaration");
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind(), "const");
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind(), "variable_declarator");

    // The later selector with a text regex overrides the earlier one.
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind(), "identifier");
    assert_eq!(*cursor.node_properties(), Properties { a: Some("z".to_string()), b: None });

    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind(), "=");
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind(), "call_expression");

    // The selectors with text regexes override the selector without one.
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind(), "identifier");
    assert_eq!(*cursor.node_properties(), Properties { a: Some("y".to_string()), b: None });

    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind(), "arguments");
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind(), "(");

    // This node doesn't match either of the regexes.
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind(), "identifier");
    assert_eq!(*cursor.node_properties(), Properties { a: Some("x".to_string()), b: None });
}
