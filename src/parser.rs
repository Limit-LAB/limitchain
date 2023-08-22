#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Parser {
    regex: String,
    taking_index: Option<Vec<usize>>,
    taking_group: Option<Vec<String>>,
}

impl Parser {
    pub fn parse(&self, input: &str) -> Option<Vec<String>> {
        let re = regex::Regex::new(&self.regex).unwrap();
        let caps = re.captures(input)?;
        let mut result = Vec::new();
        if let Some(taking_index) = &self.taking_index {
            for index in taking_index {
                result.push(caps.get(*index)?.as_str().to_string());
            }
        } else if let Some(taking_group) = &self.taking_group {
            for group in taking_group {
                result.push(caps.name(group)?.as_str().to_string());
            }
        }
        Some(result)
    }
}

#[test]
fn test_parser_serialized_simple() {
    // simple example by index
    let parser_json = r#"{
        "regex": "^(?P<name>[a-zA-Z0-9]+)\\s+(?P<age>[0-9]+)$",
        "taking_index": [1, 2]
    }"#;

    let to_parse = "John 42";

    let parser: Parser = serde_json::from_str(parser_json).unwrap();
    let result = parser.parse(to_parse).unwrap();
    assert_eq!(result, vec!["John", "42"]);

    // simple example by group
    let parser_json = r#"{
            "regex": "^(?P<name>[a-zA-Z0-9]+)\\s+(?P<age>[0-9]+)$",
            "taking_group": ["name", "age"]
        }"#;

    let to_parse = "John 42";

    let parser: Parser = serde_json::from_str(parser_json).unwrap();
    let result = parser.parse(to_parse).unwrap();
    assert_eq!(result, vec!["John", "42"]);
}

#[test]
fn test_parser_serialized_realworld() {
    // more complex example
    // spliting ','
    let parser_json = r#"{
        "regex": "Action: (?P<ACTION>.*?)\nAction Input: (?P<ACTION_INPUT>.*)",
        "taking_group": ["ACTION", "ACTION_INPUT"]
    }"#;
    let to_parse = r"
Thought: Do I need to use a tool? Yes
Action: the action to take, should be one of [{tool_names}]
Action Input: the input to the action
Observation: the result of the action
";
    let parser: Parser = serde_json::from_str(parser_json).unwrap();
    let result = parser.parse(to_parse);
    println!("{:?}", result);
}
