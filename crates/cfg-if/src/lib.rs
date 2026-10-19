//! Kinogaki's own `cfg-if`, published under the crates.io name so the frozen
//! naga, wgpu, wgpu-hal, blake3 and libloading resolve to it through
//! `[patch.crates-io]`. It is not upstream cfg-if.
//!
//! [`cfg_if!`] takes an `if #[cfg(a)] { .. } else if #[cfg(b)] { .. } else { .. }`
//! chain and keeps the tokens of the first branch whose predicate holds: each
//! branch is emitted as a nested macro call under its own predicate and the
//! negation of every earlier one, so the branches may hold items, statements
//! or a block's tail expression.

#[macro_export]
macro_rules! cfg_if {
    // The whole chain: collect (predicate, tokens) pairs, then emit.
    (
        if #[cfg($($first:tt)*)] { $($first_tokens:tt)* }
        $(else if #[cfg($($rest:tt)*)] { $($rest_tokens:tt)* })*
        $(else { $($last_tokens:tt)* })?
    ) => {
        $crate::cfg_if! {
            @emit ();
            ( ($($first)*) ($($first_tokens)*) )
            $( ( ($($rest)*) ($($rest_tokens)*) ) )*
            $( ( (all()) ($($last_tokens)*) ) )?
        }
    };
    // Emit one branch under its predicate and not(any(earlier)), then recurse
    // with its predicate added to the earlier ones.
    (@emit ($($earlier:tt)*); ( ($($pred:tt)*) ($($tokens:tt)*) ) $($tail:tt)*) => {
        #[cfg(all($($pred)*, not(any($($earlier)*))))]
        $crate::cfg_if! { @tokens $($tokens)* }
        $crate::cfg_if! { @emit ($($earlier)* $($pred)*,); $($tail)* }
    };
    (@emit ($($earlier:tt)*);) => {};
    (@tokens $($tokens:tt)*) => {
        $($tokens)*
    };
}
