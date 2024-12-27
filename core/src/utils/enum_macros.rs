use crate::imports::*;

#[macro_export]
macro_rules! define_indexed_enum {
  // Base case: define the enum
  (@enum $name:ident { $($variant:ident = $value:expr),* $(,)? }) => {
    use num_derive::{FromPrimitive, ToPrimitive};
    use num_traits::{FromPrimitive, ToPrimitive};
    #[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive, ToPrimitive)]
    pub enum $name {
      None = 0,
      $($variant = $value),*
    }
  };

  // Recursive case: build up the enum definition
  (@build $name:ident { $($variant:ident = $value:expr),* }, $n:expr, $next:ident $(, $rest:ident)*) => {
    define_indexed_enum!(@build $name { $($variant = $value,)* $next = $n }, $n + 1, $($rest),*);
  };

  // Base case for recursion: all variants processed
  (@build $name:ident { $($variant:ident = $value:expr),* }, $n:expr,) => {
    define_indexed_enum!(@enum $name { $($variant = $value),* });
  };

  // Entry point
  ($name:ident, $($variant:ident),+) => {
    define_indexed_enum!(@build $name {}, 1, $($variant),+);
  };
}

pub(crate) use define_indexed_enum;