use itertools::Itertools;

use super::Splitter;

pub struct RecursiveCharacterSplitter {
    split_by: Vec<String>,
}

impl Default for RecursiveCharacterSplitter {
    fn default() -> Self {
        RecursiveCharacterSplitter {
            split_by: vec![
                "\n\n".to_string(),
                "\n".to_string(),
                ".".to_string(),
                "。".to_string(),
                " ".to_string(),
            ],
        }
    }
}

// 函数default_splitter，用于将字符串text按照len的长度，overlapping重叠量进行分割，返回一个字符串数组
fn default_splitter(text: String, len: usize, overlapping: usize) -> Vec<String> {
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

impl Splitter for RecursiveCharacterSplitter {
    fn split(&self, text: String, len: usize, overlapping: usize) -> Vec<String> {
        assert!(overlapping < len);
        let mut split_by = self.split_by.clone();
        let splter = split_by.pop().unwrap();
        let split = text.split(&splter).map(|s| s.to_string()).collect_vec();
        split
            .into_iter()
            .flat_map(|s| {
                if s.len() <= len {
                    vec![s.trim().to_string()]
                } else {
                    if split_by.len() == 0 {
                        default_splitter(s, len, overlapping)
                    } else {
                        let splitter = RecursiveCharacterSplitter {
                            split_by: split_by.clone(),
                        };
                        splitter.split(s, len, overlapping)
                    }
                }
            })
            .collect_vec()
    }
}

#[test]
fn test_splitter() {
    let doc = "
默认的文本拆分器是RecursiveCharacterTextSplitter。这个文本拆分器会将一系列的字符作为输入。它试着根据第一个字符来分割文本，如果某个文本块太大了，就会尝试用后面的字符来分割。默认情况下，它会尝试用 这四个字符来分割文本。
除了控制可分割的字符之外，你还可以控制一些其他的东西：
length_function：用于计算文本块长度的方法。默认只是简单的计算字符数，但是在这里传递一个token计数器是非常常见的。
chunk_size：文本块的最大尺寸（由长度函数衡量）。
chunk_overlap：文本块之间的最大重叠量。保留一些重叠可以保持文本块之间的连续性（例如使用滑动窗口）。 ps.可以想象一下上学的时候，有经验的老师都会在上新课前带着同学们回顾一下上节课学到的知识，做一个承上启下。
下面是一个使用RecursiveCharacterTextSplitter拆分长文本的例子：
    ";

    let splitter = RecursiveCharacterSplitter {
        split_by: vec![
            "\n\n".to_string(),
            "\n".to_string(),
            ".".to_string(),
            "。".to_string(),
        ],
    };
    let res = splitter.split(doc.to_string(), 50, 20);
    for r in res {
        println!("{}\n", r);
        assert!(r.chars().collect::<Vec<_>>().len() <= 50);
    }
}
