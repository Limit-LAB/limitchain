use std::{borrow::Cow, collections::BTreeMap, io::Write};

use itertools::Itertools;

/// A prompt template is a string with variables that can be replaced with values.
/// ** IMPORTANT: stop is a reserved variable name **
/// "a simple prompt"
/// "a simple prompt with a variable: {var}"
/// "a simple prompt with a variable: {var} and another: {var2}"
/// "a simple prompt with a partial variable: {var:"default value"}"
/// "escape brackets with a backslash: \{var\}"
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PromptTemplate {
    text_template: String,
    pub(crate) variables: BTreeMap<String, String>,
    variables_insert_positions: BTreeMap<String, usize>,
}

impl PromptTemplate {
    /// Format the prompt template with the given values.
    pub fn format(&self, values: &BTreeMap<String, String>) -> Option<String> {
        // first asserting that all variables in the values
        self.variables.iter().for_each(|(key, value)| {
            if value.is_empty() {
                assert!(
                    values.contains_key(key),
                    "missing variable {} in values",
                    key
                );
            }
        });
        let mut values = Cow::Borrowed(values);
        // then setting the default values
        self.variables.iter().for_each(|(key, value)| {
            if !value.is_empty() && !values.contains_key(key) {
                values.to_mut().insert(key.clone(), value.clone());
            }
        });

        let mut result = self.text_template.clone();
        for (k, v) in self
            .variables_insert_positions
            .iter()
            .sorted_by(|(_k1, v1), (_k2, v2)| v1.cmp(v2))
            .rev()
        {
            result.insert_str(*v, values.get(k)?);
        }

        Some(result)
    }

    /// Save a prompt template to a file.
    pub fn save(&self, path: &str) -> std::io::Result<()> {
        let mut file = std::fs::File::create(path)?;
        file.write_all(serde_json::to_string(self).unwrap().as_bytes())?;
        Ok(())
    }

    /// Load a prompt template from a file.
    pub fn load(path: &str) -> std::io::Result<Self> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let result = serde_json::from_reader(reader)?;
        Ok(result)
    }

    // 解析字符串，返回PromptTemplate
    fn parse_from_string<S: ToString>(s: S) -> PromptTemplate {
        // 将字符串转换为字符串类型
        let s = s.to_string();
        // 创建一个BTreeMap，用于存储变量
        let mut variables = BTreeMap::new();
        // 创建一个BTreeMap，用于存储变量插入位置
        let mut variables_insert_positions = BTreeMap::new();
        // 创建一个字符串，用于存储模板文本
        let mut text_template = String::new();
        // 创建一个字符串，用于存储变量名
        let mut variable_name = String::new();
        // 创建一个字符串，用于存储变量默认值
        let mut variable_default_value = String::new();
        // 创建一个布尔值，用于判断变量默认值模式
        let mut variable_default_variable_start = false;
        // 创建一个布尔值，用于判断变量默认值模式
        let mut variable_default_value_mode = false;
        // 创建一个布尔值，用于判断变量模式
        let mut variable_mode = false;
        // 创建一个布尔值，用于判断转义模式
        let mut escape_mode = false;
        // 遍历字符串中的每一个字符
        for c in s.chars() {
            // 如果处于转义模式，则将字符添加到模板文本中
            if escape_mode {
                text_template.push(c);
                escape_mode = false;
            // 如果字符是转义字符，则将转义模式设置为true
            } else if c == '\\' {
                escape_mode = true;
            // 如果处于变量默认值模式，则根据字符进行不同的操作
            } else if variable_default_value_mode {
                // 如果字符是双引号，且不是变量默认值变量开始，则将变量默认值变量开始设置为true
                if c == '"' && !variable_default_variable_start {
                    variable_default_variable_start = true;
                // 如果字符是双引号，则将变量默认值插入位置插入变量，并将变量默认值模式设置为false
                } else if c == '"' {
                    variables_insert_positions.insert(variable_name.clone(), text_template.len());
                    variables.insert(variable_name, variable_default_value);
                    variable_default_value_mode = false;
                    variable_name = String::new();
                    variable_default_value = String::new();
                // 否则，将字符添加到变量默认值中
                } else {
                    variable_default_value.push(c);
                }
            // 如果处于变量模式，则根据字符进行不同的操作
            } else if variable_mode {
                // 如果字符是冒号，则将变量默认值模式设置为true
                if c == ':' {
                    variable_default_value_mode = true;
                // 如果字符是右花括号，则将变量插入位置插入变量，并将变量模式设置为false
                } else if c == '}' {
                    if !variable_name.is_empty() {
                        variables_insert_positions
                            .insert(variable_name.clone(), text_template.len());
                        variables.insert(variable_name, String::new());
                    }
                    variable_mode = false;
                    variable_name = String::new();
                // 否则，将字符添加到变量名中
                } else {
                    variable_name.push(c);
                }
            // 如果处于变量模式，则将字符添加到模板文本中
            } else if c == '{' {
                variable_mode = true;
            // 否则，将字符添加到模板文本中
            } else {
                text_template.push(c);
            }
        }
        PromptTemplate {
            text_template,
            variables,
            variables_insert_positions,
        }
    }
}

impl From<String> for PromptTemplate {
    fn from(s: String) -> Self {
        PromptTemplate::parse_from_string(s)
    }
}

#[test]
fn test_prompt_template() {
    let templates = [
        "a simple prompt",
        "a simple prompt with a variable: {var}",
        "a simple prompt with a variable: {var} and another: {var2}",
        "a simple prompt with a partial variable: {var:\"default value\"}",
        "escape brackets with a backslash: \\{var\\}",
    ];

    let result = [
        BTreeMap::new(),
        [("var".to_string(), String::new())]
            .iter()
            .cloned()
            .collect(),
        [
            ("var".to_string(), String::new()),
            ("var2".to_string(), String::new()),
        ]
        .iter()
        .cloned()
        .collect(),
        [("var".to_string(), "default value".to_string())]
            .iter()
            .cloned()
            .collect(),
        BTreeMap::new(),
    ];

    for (i, template) in templates.iter().enumerate() {
        let parsed = PromptTemplate::from(template.to_string());
        println!("template: {:?}", parsed);
        assert_eq!(parsed.variables, result[i]);
    }
}

#[test]
fn test_prompt_format() {
    let templates = [
        "a simple prompt",
        "a simple prompt with a variable: {var}",
        "a simple prompt with a variable: {var} and another: {var2}",
        "a simple prompt with a partial variable: {var:\"default value\"}",
        "escape brackets with a backslash: \\{var\\}",
    ];
    let vars = vec![
        BTreeMap::new(),
        [("var".to_string(), "value".to_string())]
            .iter()
            .cloned()
            .collect(),
        [
            ("var".to_string(), "value".to_string()),
            ("var2".to_string(), "value2".to_string()),
        ]
        .iter()
        .cloned()
        .collect(),
        BTreeMap::new(),
        BTreeMap::new(),
    ];
    let result = [
        "a simple prompt",
        "a simple prompt with a variable: value",
        "a simple prompt with a variable: value and another: value2",
        "a simple prompt with a partial variable: default value",
        "escape brackets with a backslash: {var}",
    ];
    for (i, template) in templates.iter().enumerate() {
        let parsed = PromptTemplate::from(template.to_string());
        let formatted = parsed.format(&vars[i]);
        println!("template: {:?}", parsed);
        println!("formatted: {:?}", formatted);
        assert_eq!(formatted, Some(result[i].to_string()));
    }
}

#[test]
fn test_save_load() {
    let prompt = "a simple prompt with a partial variable: {var:\"default value\"}";
    let parsed = PromptTemplate::from(prompt.to_string());
    parsed.save("test.json").unwrap();
    let loaded = PromptTemplate::load("test.json").unwrap();
    assert_eq!(parsed, loaded);

    // cleanup
    std::fs::remove_file("test.json").unwrap();
}
