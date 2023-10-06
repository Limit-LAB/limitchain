pub mod recursive_character_splitter;

pub trait Splitter {
    fn split(&self, text: String, len: usize, overlapping: usize) -> Vec<String>;
}
