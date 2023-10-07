use itertools::Itertools;

use super::loader::Document;

pub mod markdown_splitter;
pub mod recursive_character_splitter;

// 函数default_splitter，用于将字符串text按照len的长度，overlapping重叠量进行分割，返回一个字符串数组
pub fn default_splitter(text: String, len: usize, overlapping: usize) -> Vec<String> {
    // 如果text的长度小于len，则直接返回一个字符串数组，其中包含text
    if text.len() <= len {
        return vec![text];
    }

    // 将text转换为字符数组
    let text = text.chars().collect::<Vec<char>>();
    // 初始化一个字符串数组，用于存放分割后的字符串
    let mut result = Vec::new();
    // 初始化一个变量start，用于记录当前分割的起始位置
    let mut start = 0;

    // 当start + len小于text的长度时，循环执行以下操作
    while start + len <= text.len() {
        // 计算当前分割的结束位置
        let end = start + len;
        // 获取当前分割的字符串
        let substring = &text[start..end];
        // 将当前分割的字符串添加到字符串数组中
        result.push(substring.iter().collect::<String>());
        // 更新start的值，使其重新开始分割
        start += len - overlapping;
    }

    // 当start小于text的长度时，循环执行以下操作
    if start < text.len() {
        // 获取剩余的字符串
        let substring = &text[start..];
        // 将剩余的字符串添加到字符串数组中
        result.push(substring.iter().collect::<String>());
    }

    // 返回字符串数组
    result
}

pub trait Splitter {
    fn split(&self, text: String, len: usize, overlapping: usize) -> Vec<String> {
        default_splitter(text, len, overlapping)
    }
    fn split_docs(&self, docs: Vec<Document>, len: usize, overlapping: usize) -> Vec<Document> {
        docs.into_iter()
            .map(|d| vec![d])
            .fold(vec![], |mut d1, d2| {
                if d1.len() == 0 {
                    d2
                } else {
                    if d1.last().unwrap().text.len() + d2.first().unwrap().text.len()
                        <= len + overlapping
                    {
                        let d1last = d1.last_mut().unwrap();
                        d1last.text += &d2.first().unwrap().text;
                        d1last
                            .meta
                            .as_object_mut()
                            .unwrap()
                            .insert("merge".to_string(), d2.first().unwrap().meta.clone());
                        d1
                    } else {
                        d1.push(d2.first().unwrap().clone());
                        d1
                    }
                }
            })
            .into_iter()
            .flat_map(|d| {
                if d.text.len() <= len {
                    vec![d]
                } else {
                    self.split(d.text, len, overlapping)
                        .into_iter()
                        .map(|s| Document {
                            text: s,
                            meta: d.meta.clone(),
                        })
                        .collect()
                }
            })
            .collect_vec()
    }
}
