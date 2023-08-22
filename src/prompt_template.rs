use std::{borrow::Cow, collections::BTreeMap, io::Write};

use itertools::Itertools;

/// A prompt template is a string with variables that can be replaced with values.
/// "a simple prompt"
/// "a simple prompt with a variable: {var}"
/// "a simple prompt with a variable: {var} and another: {var2}"
/// "a simple prompt with a partial variable: {var:"default value"}"
/// "escape brackets with a backslash: \{var\}"
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PromptTemplate {
    text_template: String,
    variables: BTreeMap<String, String>,
    variables_insert_positions: BTreeMap<String, usize>,
}

impl PromptTemplate {
    /// Format the prompt template with the given values.
    pub fn format(&self, values: &BTreeMap<String, String>) -> String {
        // first asserting that all variables in the values
        self.variables.iter().for_each(|(key, value)| {
            if value == "" {
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
            if value != "" {
                if !values.contains_key(key) {
                    values.to_mut().insert(key.clone(), value.clone());
                }
            }
        });

        let mut result = self.text_template.clone();
        for (k, v) in self
            .variables_insert_positions
            .iter()
            .sorted_by(|(k1, v1), (k2, v2)| v1.cmp(v2))
            .rev()
        {
            result.insert_str(*v, values.get(k).unwrap());
        }

        result
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

    fn parse_from_string<S: ToString>(s: S) -> PromptTemplate {
        let s = s.to_string();
        let mut variables = BTreeMap::new();
        let mut variables_insert_positions = BTreeMap::new();
        let mut text_template = String::new();
        let mut variable_name = String::new();
        let mut variable_default_value = String::new();
        let mut variable_default_variable_start = false;
        let mut variable_default_value_mode = false;
        let mut variable_mode = false;
        let mut escape_mode = false;
        for c in s.chars() {
            if escape_mode {
                text_template.push(c);
                escape_mode = false;
            } else if c == '\\' {
                escape_mode = true;
            } else if variable_default_value_mode {
                if c == '"' && !variable_default_variable_start {
                    variable_default_variable_start = true;
                } else if c == '"' {
                    variables_insert_positions.insert(variable_name.clone(), text_template.len());
                    variables.insert(variable_name, variable_default_value);
                    variable_default_value_mode = false;
                    variable_name = String::new();
                    variable_default_value = String::new();
                } else {
                    variable_default_value.push(c);
                }
            } else if variable_mode {
                if c == ':' {
                    variable_default_value_mode = true;
                } else if c == '}' {
                    if !variable_name.is_empty() {
                        variables_insert_positions
                            .insert(variable_name.clone(), text_template.len());
                        variables.insert(variable_name, String::new());
                    }
                    variable_mode = false;
                    variable_name = String::new();
                } else {
                    variable_name.push(c);
                }
            } else if c == '{' {
                variable_mode = true;
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
    let templates = vec![
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
    let templates = vec![
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
    let result = vec![
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
        assert_eq!(formatted, result[i]);
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