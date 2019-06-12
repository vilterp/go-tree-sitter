use super::helpers::fixtures::{get_language, get_test_language};
use super::helpers::random::Rand;
use super::helpers::edits::{get_random_edit, perform_edit};
use crate::generate::generate_parser_for_grammar;
use tree_sitter::{Node, Parser, Point, Tree};

const JSON_EXAMPLE: &'static str = r#"

[
  123,
  false,
  {
    "x": null
  }
]
"#;

const GRAMMAR_WITH_ALIASES_AND_EXTRAS: &'static str = r#"{
  "name": "aliases_and_extras",

  "extras": [
    {"type": "PATTERN", "value": "\\s+"},
    {"type": "SYMBOL", "name": "comment"}
  ],

  "rules": {
    "a": {
      "type": "SEQ",
      "members": [
        {"type": "SYMBOL", "name": "b"},
        {
          "type": "ALIAS",
          "value": "B",
          "named": true,
          "content": {"type": "SYMBOL", "name": "b"}
        },
        {
          "type": "ALIAS",
          "value": "C",
          "named": true,
          "content": {"type": "SYMBOL", "name": "_c"}
        }
      ]
    },

    "b": {"type": "STRING", "value": "b"},

    "_c": {"type": "STRING", "value": "c"},

    "comment": {"type": "STRING", "value": "..."}
  }
}"#;

#[test]
fn test_node_child() {
    let tree = parse_json_example();
    let array_node = tree.root_node().child(0).unwrap();

    assert_eq!(array_node.kind(), "array");
    assert_eq!(array_node.named_child_count(), 3);
    assert_eq!(array_node.start_byte(), JSON_EXAMPLE.find("[").unwrap());
    assert_eq!(array_node.end_byte(), JSON_EXAMPLE.find("]").unwrap() + 1);
    assert_eq!(array_node.start_position(), Point::new(2, 0));
    assert_eq!(array_node.end_position(), Point::new(8, 1));
    assert_eq!(array_node.child_count(), 7);

    let left_bracket_node = array_node.child(0).unwrap();
    let number_node = array_node.child(1).unwrap();
    let comma_node1 = array_node.child(2).unwrap();
    let false_node = array_node.child(3).unwrap();
    let comma_node2 = array_node.child(4).unwrap();
    let object_node = array_node.child(5).unwrap();
    let right_bracket_node = array_node.child(6).unwrap();

    assert_eq!(left_bracket_node.kind(), "[");
    assert_eq!(number_node.kind(), "number");
    assert_eq!(comma_node1.kind(), ",");
    assert_eq!(false_node.kind(), "false");
    assert_eq!(comma_node2.kind(), ",");
    assert_eq!(object_node.kind(), "object");
    assert_eq!(right_bracket_node.kind(), "]");

    assert_eq!(left_bracket_node.is_named(), false);
    assert_eq!(number_node.is_named(), true);
    assert_eq!(comma_node1.is_named(), false);
    assert_eq!(false_node.is_named(), true);
    assert_eq!(comma_node2.is_named(), false);
    assert_eq!(object_node.is_named(), true);
    assert_eq!(right_bracket_node.is_named(), false);

    assert_eq!(number_node.start_byte(), JSON_EXAMPLE.find("123").unwrap());
    assert_eq!(
        number_node.end_byte(),
        JSON_EXAMPLE.find("123").unwrap() + 3
    );
    assert_eq!(number_node.start_position(), Point::new(3, 2));
    assert_eq!(number_node.end_position(), Point::new(3, 5));

    assert_eq!(false_node.start_byte(), JSON_EXAMPLE.find("false").unwrap());
    assert_eq!(
        false_node.end_byte(),
        JSON_EXAMPLE.find("false").unwrap() + 5
    );
    assert_eq!(false_node.start_position(), Point::new(4, 2));
    assert_eq!(false_node.end_position(), Point::new(4, 7));

    assert_eq!(object_node.start_byte(), JSON_EXAMPLE.find("{").unwrap());
    assert_eq!(object_node.start_position(), Point::new(5, 2));
    assert_eq!(object_node.end_position(), Point::new(7, 3));

    assert_eq!(object_node.child_count(), 3);
    let left_brace_node = object_node.child(0).unwrap();
    let pair_node = object_node.child(1).unwrap();
    let right_brace_node = object_node.child(2).unwrap();

    assert_eq!(left_brace_node.kind(), "{");
    assert_eq!(pair_node.kind(), "pair");
    assert_eq!(right_brace_node.kind(), "}");

    assert_eq!(left_brace_node.is_named(), false);
    assert_eq!(pair_node.is_named(), true);
    assert_eq!(right_brace_node.is_named(), false);

    assert_eq!(pair_node.start_byte(), JSON_EXAMPLE.find("\"x\"").unwrap());
    assert_eq!(pair_node.end_byte(), JSON_EXAMPLE.find("null").unwrap() + 4);
    assert_eq!(pair_node.start_position(), Point::new(6, 4));
    assert_eq!(pair_node.end_position(), Point::new(6, 13));

    assert_eq!(pair_node.child_count(), 3);
    let string_node = pair_node.child(0).unwrap();
    let colon_node = pair_node.child(1).unwrap();
    let null_node = pair_node.child(2).unwrap();

    assert_eq!(string_node.kind(), "string");
    assert_eq!(colon_node.kind(), ":");
    assert_eq!(null_node.kind(), "null");

    assert_eq!(string_node.is_named(), true);
    assert_eq!(colon_node.is_named(), false);
    assert_eq!(null_node.is_named(), true);

    assert_eq!(
        string_node.start_byte(),
        JSON_EXAMPLE.find("\"x\"").unwrap()
    );
    assert_eq!(
        string_node.end_byte(),
        JSON_EXAMPLE.find("\"x\"").unwrap() + 3
    );
    assert_eq!(string_node.start_position(), Point::new(6, 4));
    assert_eq!(string_node.end_position(), Point::new(6, 7));

    assert_eq!(null_node.start_byte(), JSON_EXAMPLE.find("null").unwrap());
    assert_eq!(null_node.end_byte(), JSON_EXAMPLE.find("null").unwrap() + 4);
    assert_eq!(null_node.start_position(), Point::new(6, 9));
    assert_eq!(null_node.end_position(), Point::new(6, 13));

    assert_eq!(string_node.parent().unwrap(), pair_node);
    assert_eq!(null_node.parent().unwrap(), pair_node);
    assert_eq!(pair_node.parent().unwrap(), object_node);
    assert_eq!(number_node.parent().unwrap(), array_node);
    assert_eq!(false_node.parent().unwrap(), array_node);
    assert_eq!(object_node.parent().unwrap(), array_node);
    assert_eq!(array_node.parent().unwrap(), tree.root_node());
    assert_eq!(tree.root_node().parent(), None);
}

#[test]
fn test_node_named_child() {
    let tree = parse_json_example();
    let array_node = tree.root_node().child(0).unwrap();

    let number_node = array_node.named_child(0).unwrap();
    let false_node = array_node.named_child(1).unwrap();
    let object_node = array_node.named_child(2).unwrap();

    assert_eq!(number_node.kind(), "number");
    assert_eq!(number_node.start_byte(), JSON_EXAMPLE.find("123").unwrap());
    assert_eq!(
        number_node.end_byte(),
        JSON_EXAMPLE.find("123").unwrap() + 3
    );
    assert_eq!(number_node.start_position(), Point::new(3, 2));
    assert_eq!(number_node.end_position(), Point::new(3, 5));

    assert_eq!(false_node.kind(), "false");
    assert_eq!(false_node.start_byte(), JSON_EXAMPLE.find("false").unwrap());
    assert_eq!(
        false_node.end_byte(),
        JSON_EXAMPLE.find("false").unwrap() + 5
    );
    assert_eq!(false_node.start_position(), Point::new(4, 2));
    assert_eq!(false_node.end_position(), Point::new(4, 7));

    assert_eq!(object_node.kind(), "object");
    assert_eq!(object_node.start_byte(), JSON_EXAMPLE.find("{").unwrap());
    assert_eq!(object_node.start_position(), Point::new(5, 2));
    assert_eq!(object_node.end_position(), Point::new(7, 3));

    assert_eq!(object_node.named_child_count(), 1);

    let pair_node = object_node.named_child(0).unwrap();
    assert_eq!(pair_node.kind(), "pair");
    assert_eq!(pair_node.start_byte(), JSON_EXAMPLE.find("\"x\"").unwrap());
    assert_eq!(pair_node.end_byte(), JSON_EXAMPLE.find("null").unwrap() + 4);
    assert_eq!(pair_node.start_position(), Point::new(6, 4));
    assert_eq!(pair_node.end_position(), Point::new(6, 13));

    let string_node = pair_node.named_child(0).unwrap();
    let null_node = pair_node.named_child(1).unwrap();

    assert_eq!(string_node.kind(), "string");
    assert_eq!(null_node.kind(), "null");

    assert_eq!(
        string_node.start_byte(),
        JSON_EXAMPLE.find("\"x\"").unwrap()
    );
    assert_eq!(
        string_node.end_byte(),
        JSON_EXAMPLE.find("\"x\"").unwrap() + 3
    );
    assert_eq!(string_node.start_position(), Point::new(6, 4));
    assert_eq!(string_node.end_position(), Point::new(6, 7));

    assert_eq!(null_node.start_byte(), JSON_EXAMPLE.find("null").unwrap());
    assert_eq!(null_node.end_byte(), JSON_EXAMPLE.find("null").unwrap() + 4);
    assert_eq!(null_node.start_position(), Point::new(6, 9));
    assert_eq!(null_node.end_position(), Point::new(6, 13));

    assert_eq!(string_node.parent().unwrap(), pair_node);
    assert_eq!(null_node.parent().unwrap(), pair_node);
    assert_eq!(pair_node.parent().unwrap(), object_node);
    assert_eq!(number_node.parent().unwrap(), array_node);
    assert_eq!(false_node.parent().unwrap(), array_node);
    assert_eq!(object_node.parent().unwrap(), array_node);
    assert_eq!(array_node.parent().unwrap(), tree.root_node());
    assert_eq!(tree.root_node().parent(), None);
}

#[test]
fn test_node_named_child_with_aliases_and_extras() {
    let (parser_name, parser_code) =
        generate_parser_for_grammar(GRAMMAR_WITH_ALIASES_AND_EXTRAS).unwrap();

    let mut parser = Parser::new();
    parser
        .set_language(get_test_language(&parser_name, &parser_code, None))
        .unwrap();

    let tree = parser.parse("b ... b ... c", None).unwrap();
    let root = tree.root_node();
    assert_eq!(root.to_sexp(), "(a (b) (comment) (B) (comment) (C))");
    assert_eq!(root.named_child_count(), 5);
    assert_eq!(root.named_child(0).unwrap().kind(), "b");
    assert_eq!(root.named_child(1).unwrap().kind(), "comment");
    assert_eq!(root.named_child(2).unwrap().kind(), "B");
    assert_eq!(root.named_child(3).unwrap().kind(), "comment");
    assert_eq!(root.named_child(4).unwrap().kind(), "C");
}

#[test]
fn test_node_descendant_for_range() {
    let tree = parse_json_example();
    let array_node = tree.root_node().child(0).unwrap();

    // Leaf node starts and ends at the given bounds - byte query
    let colon_index = JSON_EXAMPLE.find(":").unwrap();
    let colon_node = array_node
        .descendant_for_byte_range(colon_index, colon_index + 1)
        .unwrap();
    assert_eq!(colon_node.kind(), ":");
    assert_eq!(colon_node.start_byte(), colon_index);
    assert_eq!(colon_node.end_byte(), colon_index + 1);
    assert_eq!(colon_node.start_position(), Point::new(6, 7));
    assert_eq!(colon_node.end_position(), Point::new(6, 8));

    // Leaf node starts and ends at the given bounds - point query
    let colon_node = array_node
        .descendant_for_point_range(Point::new(6, 7), Point::new(6, 8))
        .unwrap();
    assert_eq!(colon_node.kind(), ":");
    assert_eq!(colon_node.start_byte(), colon_index);
    assert_eq!(colon_node.end_byte(), colon_index + 1);
    assert_eq!(colon_node.start_position(), Point::new(6, 7));
    assert_eq!(colon_node.end_position(), Point::new(6, 8));

    // Leaf node starts at the lower bound, ends after the upper bound - byte query
    let string_index = JSON_EXAMPLE.find("\"x\"").unwrap();
    let string_node = array_node
        .descendant_for_byte_range(string_index, string_index + 2)
        .unwrap();
    assert_eq!(string_node.kind(), "string");
    assert_eq!(string_node.start_byte(), string_index);
    assert_eq!(string_node.end_byte(), string_index + 3);
    assert_eq!(string_node.start_position(), Point::new(6, 4));
    assert_eq!(string_node.end_position(), Point::new(6, 7));

    // Leaf node starts at the lower bound, ends after the upper bound - point query
    let string_node = array_node
        .descendant_for_point_range(Point::new(6, 4), Point::new(6, 6))
        .unwrap();
    assert_eq!(string_node.kind(), "string");
    assert_eq!(string_node.start_byte(), string_index);
    assert_eq!(string_node.end_byte(), string_index + 3);
    assert_eq!(string_node.start_position(), Point::new(6, 4));
    assert_eq!(string_node.end_position(), Point::new(6, 7));

    // Leaf node starts before the lower bound, ends at the upper bound - byte query
    let null_index = JSON_EXAMPLE.find("null").unwrap();
    let null_node = array_node
        .descendant_for_byte_range(null_index + 1, null_index + 4)
        .unwrap();
    assert_eq!(null_node.kind(), "null");
    assert_eq!(null_node.start_byte(), null_index);
    assert_eq!(null_node.end_byte(), null_index + 4);
    assert_eq!(null_node.start_position(), Point::new(6, 9));
    assert_eq!(null_node.end_position(), Point::new(6, 13));

    // Leaf node starts before the lower bound, ends at the upper bound - point query
    let null_node = array_node
        .descendant_for_point_range(Point::new(6, 11), Point::new(6, 13))
        .unwrap();
    assert_eq!(null_node.kind(), "null");
    assert_eq!(null_node.start_byte(), null_index);
    assert_eq!(null_node.end_byte(), null_index + 4);
    assert_eq!(null_node.start_position(), Point::new(6, 9));
    assert_eq!(null_node.end_position(), Point::new(6, 13));

    // The bounds span multiple leaf nodes - return the smallest node that does span it.
    let pair_node = array_node
        .descendant_for_byte_range(string_index + 2, string_index + 4)
        .unwrap();
    assert_eq!(pair_node.kind(), "pair");
    assert_eq!(pair_node.start_byte(), string_index);
    assert_eq!(pair_node.end_byte(), string_index + 9);
    assert_eq!(pair_node.start_position(), Point::new(6, 4));
    assert_eq!(pair_node.end_position(), Point::new(6, 13));

    assert_eq!(colon_node.parent(), Some(pair_node));

    // no leaf spans the given range - return the smallest node that does span it.
    let pair_node = array_node
        .named_descendant_for_point_range(Point::new(6, 6), Point::new(6, 8))
        .unwrap();
    assert_eq!(pair_node.kind(), "pair");
    assert_eq!(pair_node.start_byte(), string_index);
    assert_eq!(pair_node.end_byte(), string_index + 9);
    assert_eq!(pair_node.start_position(), Point::new(6, 4));
    assert_eq!(pair_node.end_position(), Point::new(6, 13));
}

#[test]
fn test_node_edit() {
    let mut code = JSON_EXAMPLE.as_bytes().to_vec();
    let mut tree = parse_json_example();
    let mut rand = Rand::new(0);

    for _ in 0..10 {
        let mut nodes_before = get_all_nodes(&tree);

        let edit = get_random_edit(&mut rand, &mut code);
        let mut tree2 = tree.clone();
        let edit = perform_edit(&mut tree2, &mut code, &edit);
        for node in nodes_before.iter_mut() {
            node.edit(&edit);
        }

        let nodes_after = get_all_nodes(&tree2);
        for (i, node) in nodes_before.into_iter().enumerate() {
            assert_eq!(
                (
                    node.kind(),
                    node.start_byte(),
                    node.start_position()
                ),
                (
                    nodes_after[i].kind(),
                    nodes_after[i].start_byte(),
                    nodes_after[i].start_position()
                ),
            );
        }

        tree = tree2;
    }
}

fn get_all_nodes(tree: &Tree) -> Vec<Node> {
    let mut result = Vec::new();
    let mut visited_children = false;
    let mut cursor = tree.walk();
    loop {
        result.push(cursor.node());
        if !visited_children && cursor.goto_first_child() {
            continue;
        } else if cursor.goto_next_sibling() {
            visited_children = false;
        } else if cursor.goto_parent() {
            visited_children = true;
        } else {
            break;
        }
    }
    return result;
}

fn parse_json_example() -> Tree {
    let mut parser = Parser::new();
    parser.set_language(get_language("json")).unwrap();
    parser.parse(JSON_EXAMPLE, None).unwrap()
}
