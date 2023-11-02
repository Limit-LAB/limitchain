use itertools::Itertools;

use crate::document::splitter::default_splitter;

use super::Splitter;

pub struct RecursiveCharacterSplitter {
    pub split_by: Vec<String>,
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

impl Splitter for RecursiveCharacterSplitter {
    fn split(&self, text: String, len: usize, overlapping: usize) -> Vec<String> {
        // TODO: 实现一个递归的splitter
        // 挨个split一遍
        // 选取合适的
        // merge回去

        text.chars().chunks(len).into_iter().map(|x| x.collect::<String>()).collect_vec()
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
    let res = splitter.split(doc.to_string(), 100, 40);
    for r in res {
        println!("{}\n", r);
        assert!(r.chars().collect::<Vec<_>>().len() <= 100);
    }
}
