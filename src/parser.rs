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

pub fn unescape(s: &str) -> anyhow::Result<String> {
    // 声明一个变量in_single_quote，用于判断是否处于单引号中
    let mut in_single_quote = false;
    // 声明一个变量in_double_quote，用于判断是否处于双引号中
    let mut in_double_quote = false;
    // 声明一个变量chars，用于获取字符串s中的每一个字符
    let mut chars = s.chars().enumerate();
    // 声明一个变量res，用于存放转换后的字符串
    let mut res = String::with_capacity(s.len());

    // 遍历字符串s中的每一个字符
    while let Some((_idx, c)) = chars.next() {
        // 当处于单引号中时，不存在转义字符
        if in_single_quote {
            if c == '\'' {
                // 当遇到单引号时，将in_single_quote设置为false，并继续遍历
                in_single_quote = false;
                continue;
            }
        } else if in_double_quote {
            // 当处于双引号中时，不存在转义字符
            if c == '"' {
                // 当遇到双引号时，将in_double_quote设置为false，并继续遍历
                in_double_quote = false;
                continue;
            }

            // 当处于双引号中时，如果遇到转义字符，则进行转换
            if c == '\\' {
                match chars.next() {
                    None => {
                        // 如果字符串s以转义字符结尾，则返回错误
                        return Err(anyhow::anyhow!(
                            "UnescapeError: string ends with a single backslash"
                        ));
                    }
                    Some((idx, c2)) => {
                        // 将转义字符转换为对应的字符，并添加到res中
                        res.push(match c2 {
                            'a' => '\u{07}',
                            'b' => '\u{08}',
                            'v' => '\u{0B}',
                            'f' => '\u{0C}',
                            'n' => '\n',
                            'r' => '\r',
                            't' => '\t',
                            'e' | 'E' => '\u{1B}',
                            '\\' => '\\',
                            '\'' => '\'',
                            '"' => '"',
                            '$' => '$',
                            '`' => '`',
                            ' ' => ' ',
                            'u' => parse_unicode(&mut chars)
                                .map_err(|x| anyhow::anyhow!("UnescapeError: {}", x))?,
                            _ => {
                                // 如果遇到不合法的转义字符，则返回错误
                                return Err(anyhow::anyhow!(
                                    "UnescapeError: invalid escape character: {}",
                                    c2
                                ));
                            }
                        });
                        continue;
                    }
                };
            }
        } else if c == '\'' {
            // 当处于单引号中时，遇到单引号，则将in_single_quote设置为true，并继续遍历
            in_single_quote = true;
            continue;
        } else if c == '"' {
            // 当处于双引号中时，遇到双引号，则将in_double_quote设置为true，并继续遍历
            in_double_quote = true;
            continue;
        }

        // 将字符c添加到res中
        res.push(c);
    }

    // 返回转换后的字符串
    Ok(res)
}

fn parse_unicode<I>(chars: &mut I) -> anyhow::Result<char>
where
    I: Iterator<Item = (usize, char)>,
{
    match chars.next() {
        Some((_, '{')) => {}
        _ => {
            return Err(anyhow::anyhow!("ParseUnicodeError: brace not found"));
        }
    }

    let unicode_seq: String = chars
        .take_while(|&(_, c)| c != '}')
        .map(|(_, c)| c)
        .collect();

    u32::from_str_radix(&unicode_seq, 16)
        .map_err(|e| anyhow::anyhow!("ParseUnicodeError: {}", e))
        .and_then(|u| {
            char::from_u32(u).ok_or_else(|| anyhow::anyhow!("ParseUnicodeError: invalid unicode"))
        })
}
