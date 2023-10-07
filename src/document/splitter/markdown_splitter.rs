use crate::document::loader::{markdown::MarkdownLoader, DocumentLoader};

use super::{recursive_character_splitter::RecursiveCharacterSplitter, Splitter};

/// specialized recursive character splitter for markdown
pub struct MarkdownSplitter {
    splitter: RecursiveCharacterSplitter,
}

impl Default for MarkdownSplitter {
    fn default() -> Self {
        MarkdownSplitter {
            splitter: RecursiveCharacterSplitter {
                split_by: vec![
                    "#".to_string(),
                    "```".to_string(),
                    ">".to_string(),
                    "---".to_string(),
                    "===".to_string(),
                    "\n\n".to_string(),
                ],
            },
        }
    }
}

impl Splitter for MarkdownSplitter {
    fn split(&self, text: String, len: usize, overlapping: usize) -> Vec<String> {
        self.splitter.split(text, len, overlapping)
    }
}

#[test]
fn test_splitter() {
    let doc = "
# Recursive Character Text Splitter

默认的文本拆分器是RecursiveCharacterTextSplitter。
这个文本拆分器会将一系列的字符作为输入。
它试着根据第一个字符来分割文本，如果某个文本块太大了，就会尝试用后面的字符来分割。
默认情况下，它会尝试用 这四个字符来分割文本。

## 控制

除了控制可分割的字符之外，你还可以控制一些其他的东西：

- length_function：用于计算文本块长度的方法。默认只是简单的计算字符数，但是在这里传递一个token计数器是非常常见的。
- chunk_size：文本块的最大尺寸（由长度函数衡量）。
- chunk_overlap：文本块之间的最大重叠量。保留一些重叠可以保持文本块之间的连续性（例如使用滑动窗口）。 ps.可以想象一下上学的时候，有经验的老师都会在上新课前带着同学们回顾一下上节课学到的知识，做一个承上启下。

下面是一个使用RecursiveCharacterTextSplitter拆分长文本的例子：
    ";

    let splitter = MarkdownSplitter::default();
    let res = splitter.split(doc.to_string(), 50, 20);
    for r in res {
        println!("{}\n", r);
        assert!(r.chars().collect::<Vec<_>>().len() <= 50);
    }
}

#[test]
fn test_split_md_doc() {
    let loader = MarkdownLoader {};
    let doc = loader.load_file("test_utils/md_example.md").unwrap();

    let splitter = MarkdownSplitter::default();
    let docs = splitter.split_docs(doc, 500, 100);
    println!("{:#?}", docs);
}
