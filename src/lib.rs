#![feature(return_position_impl_trait_in_trait)]
#![feature(once_cell)]

pub mod chain;
pub mod client;
pub mod document;
pub mod parser;
pub mod prompt_template;
pub mod schema;

#[macro_export]
macro_rules! btreemap {
    ($($key:expr => $value:expr,)+) => (btreemap!($($key => $value),+));
    ($($key:expr => $value:expr),*) => {
        {
            let mut _map = ::std::collections::BTreeMap::new();
            $(
                let _ = _map.insert($key, $value);
            )*
            _map
        }
    };
}
