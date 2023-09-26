// Taken from serde_json and modified for NBT
// https://github.com/serde-rs/json/blob/829175e6069fb16672875f125f6afdd7c6da1dec/src/macros.rs#L60-L303
//
// The source uses the MIT license, which is repeated here:
//
// Permission is hereby granted, free of charge, to any
// person obtaining a copy of this software and associated
// documentation files (the "Software"), to deal in the
// Software without restriction, including without
// limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software
// is furnished to do so, subject to the following
// conditions:
//
// The above copyright notice and this permission notice
// shall be included in all copies or substantial portions
// of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
// ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
// TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
// SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
// IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

/// Produce a [`Value`][`crate::Value`] using
/// JSON/[SNBT](https://minecraft.wiki/w/NBT_format#SNBT_format)-like
/// syntax.
///
/// Example:
/// ```rust
/// use fastnbt::nbt;
/// let _ = nbt!({
///     "key1": "value1",
///     "key2": 42,
///     "key3": [4, 2],
/// });
/// ```
///
/// Unlike SNBT, key/field names for compounds need quoted strings. `"key1"`
/// above could not be simplified to just `key1`.
///
/// NBT Arrays are supported with
/// [SNBT](https://minecraft.wiki/w/NBT_format#SNBT_format) syntax:
///
/// ```rust
/// # use fastnbt::nbt;
/// let _ = nbt!({
///     "bytes": [B; 1, 2, 3],
///     "ints": [I; 1, 2, 3],
///     "longs": [L; 1, 2, 3],
/// });
/// ```
///
#[macro_export(local_inner_macros)]
macro_rules! nbt {
    // Hide distracting implementation details from the generated rustdoc.
    ($($nbt:tt)+) => {
        nbt_internal!($($nbt)+)
    };
}

#[macro_export(local_inner_macros)]
#[doc(hidden)]
macro_rules! nbt_internal {
    // Done with trailing comma.
    (@array [$($elems:expr,)*]) => {
        nbt_internal_vec![$($elems,)*]
    };

    // Done without trailing comma.
    (@array [$($elems:expr),*]) => {
        nbt_internal_vec![$($elems),*]
    };

    // Next element is an array.
    (@array [$($elems:expr,)*] [$($array:tt)*] $($rest:tt)*) => {
        nbt_internal!(@array [$($elems,)* nbt_internal!([$($array)*])] $($rest)*)
    };

    // Next element is a map.
    (@array [$($elems:expr,)*] {$($map:tt)*} $($rest:tt)*) => {
        nbt_internal!(@array [$($elems,)* nbt_internal!({$($map)*})] $($rest)*)
    };

    // Next element is an expression followed by comma.
    (@array [$($elems:expr,)*] $next:expr, $($rest:tt)*) => {
        nbt_internal!(@array [$($elems,)* nbt_internal!($next),] $($rest)*)
    };

    // Last element is an expression with no trailing comma.
    (@array [$($elems:expr,)*] $last:expr) => {
        nbt_internal!(@array [$($elems,)* nbt_internal!($last)])
    };

    // Comma after the most recent element.
    (@array [$($elems:expr),*] , $($rest:tt)*) => {
        nbt_internal!(@array [$($elems,)*] $($rest)*)
    };

    // Unexpected token after most recent element.
    (@array [$($elems:expr),*] $unexpected:tt $($rest:tt)*) => {
        nbt_unexpected!($unexpected)
    };

    /* ------------ IntArray types ------------ */

    // Done with trailing comma.
    (@int_array [$($elems:expr,)*]) => {
        nbt_internal_vec![$($elems,)*]
    };

    // Done without trailing comma.
    (@int_array [$($elems:expr),*]) => {
        nbt_internal_vec![$($elems),*]
    };

    // Next element is an expression followed by comma.
    (@int_array [$($elems:expr,)*] $next:expr, $($rest:tt)*) => {
        nbt_internal!(@int_array [$($elems,)* $next,] $($rest)*)
    };

    // Last element is an expression with no trailing comma.
    (@int_array [$($elems:expr,)*] $last:expr) => {
        nbt_internal!(@int_array [$($elems,)* $last])
    };

    // Comma after the most recent element.
    (@int_array [$($elems:expr),*] , $($rest:tt)*) => {
        nbt_internal!(@int_array [$($elems,)*] $($rest)*)
    };

    // Unexpected token after most recent element.
    (@int_array [$($elems:expr),*] $unexpected:tt $($rest:tt)*) => {
        nbt_unexpected!($unexpected)
    };

    //////////////////////////////////////////////////////////////////////////
    // TT muncher for parsing the inside of an object {...}. Each entry is
    // inserted into the given map variable.
    //
    // Must be invoked as: nbt_internal!(@object $map () ($($tt)*) ($($tt)*))
    //
    // We require two copies of the input tokens so that we can match on one
    // copy and trigger errors on the other copy.
    //////////////////////////////////////////////////////////////////////////

    // Done.
    (@object $object:ident () () ()) => {};

    // Insert the current entry followed by trailing comma.
    (@object $object:ident [$($key:tt)+] ($value:expr) , $($rest:tt)*) => {
        let _ = $object.insert(($($key)+).into(), $value);
        nbt_internal!(@object $object () ($($rest)*) ($($rest)*));
    };

    // Current entry followed by unexpected token.
    (@object $object:ident [$($key:tt)+] ($value:expr) $unexpected:tt $($rest:tt)*) => {
        nbt_unexpected!($unexpected);
    };

    // Insert the last entry without trailing comma.
    (@object $object:ident [$($key:tt)+] ($value:expr)) => {
        let _ = $object.insert(($($key)+).into(), $value);
    };

    // Next value is an array.
    (@object $object:ident ($($key:tt)+) (: [$($array:tt)*] $($rest:tt)*) $copy:tt) => {
        nbt_internal!(@object $object [$($key)+] (nbt_internal!([$($array)*])) $($rest)*);
    };

    // Next value is a map.
    (@object $object:ident ($($key:tt)+) (: {$($map:tt)*} $($rest:tt)*) $copy:tt) => {
        nbt_internal!(@object $object [$($key)+] (nbt_internal!({$($map)*})) $($rest)*);
    };

    // Next value is an expression followed by comma.
    (@object $object:ident ($($key:tt)+) (: $value:expr , $($rest:tt)*) $copy:tt) => {
        nbt_internal!(@object $object [$($key)+] (nbt_internal!($value)) , $($rest)*);
    };

    // Last value is an expression with no trailing comma.
    (@object $object:ident ($($key:tt)+) (: $value:expr) $copy:tt) => {
        nbt_internal!(@object $object [$($key)+] (nbt_internal!($value)));
    };

    // Missing value for last entry. Trigger a reasonable error message.
    (@object $object:ident ($($key:tt)+) (:) $copy:tt) => {
        // "unexpected end of macro invocation"
        nbt_internal!();
    };

    // Missing colon and value for last entry. Trigger a reasonable error
    // message.
    (@object $object:ident ($($key:tt)+) () $copy:tt) => {
        // "unexpected end of macro invocation"
        nbt_internal!();
    };

    // Misplaced colon. Trigger a reasonable error message.
    (@object $object:ident () (: $($rest:tt)*) ($colon:tt $($copy:tt)*)) => {
        // Takes no arguments so "no rules expected the token `:`".
        nbt_unexpected!($colon);
    };

    // Found a comma inside a key. Trigger a reasonable error message.
    (@object $object:ident ($($key:tt)*) (, $($rest:tt)*) ($comma:tt $($copy:tt)*)) => {
        // Takes no arguments so "no rules expected the token `,`".
        nbt_unexpected!($comma);
    };

    // Key is fully parenthesized. This avoids clippy double_parens false
    // positives because the parenthesization may be necessary here.
    (@object $object:ident () (($key:expr) : $($rest:tt)*) $copy:tt) => {
        nbt_internal!(@object $object ($key) (: $($rest)*) (: $($rest)*));
    };

    // Refuse to absorb colon token into key expression.
    (@object $object:ident ($($key:tt)*) (: $($unexpected:tt)+) $copy:tt) => {
        nbt_expect_expr_comma!($($unexpected)+);
    };

    // Munch a token into the current key.
    (@object $object:ident ($($key:tt)*) ($tt:tt $($rest:tt)*) $copy:tt) => {
        nbt_internal!(@object $object ($($key)* $tt) ($($rest)*) ($($rest)*));
    };

    //////////////////////////////////////////////////////////////////////////
    // The main implementation.
    //
    // Must be invoked as: nbt_internal!($($nbt)+)
    //////////////////////////////////////////////////////////////////////////

    ([B;]) => {
        $crate::Value::ByteArray($crate::ByteArray::new(nbt_internal_vec![]))
    };

    ([I;]) => {
        $crate::Value::IntArray($crate::IntArray::new(nbt_internal_vec![]))
    };

    ([L;]) => {
        $crate::Value::LongArray($crate::LongArray::new(nbt_internal_vec![]))
    };

    ([]) => {
        $crate::Value::List(nbt_internal_vec![])
    };

    ([B; $($tt:tt)+ ]) => {
        $crate::Value::ByteArray($crate::ByteArray::new(nbt_internal!(@int_array [] $($tt)+)))
    };

    ([I; $($tt:tt)+ ]) => {
        $crate::Value::IntArray($crate::IntArray::new(nbt_internal!(@int_array [] $($tt)+)))
    };

    ([L; $($tt:tt)+ ]) => {
        $crate::Value::LongArray($crate::LongArray::new(nbt_internal!(@int_array [] $($tt)+)))
    };

    ([ $($tt:tt)+ ]) => {
        $crate::Value::List(nbt_internal!(@array [] $($tt)+))
    };

    ({}) => {
        $crate::Value::Compound(std::collections::HashMap::new())
    };

    ({ $($tt:tt)+ }) => {
        $crate::Value::Compound({
            let mut object = std::collections::HashMap::new();
            nbt_internal!(@object object () ($($tt)+) ($($tt)+));
            object
        })
    };

    // Any Serialize type: numbers, strings, struct literals, variables etc.
    ($other:expr) => {
        $crate::to_value(&$other).unwrap()
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! nbt_unexpected {
    () => {};
}

#[macro_export]
#[doc(hidden)]
macro_rules! nbt_expect_expr_comma {
    ($e:expr , $($tt:tt)*) => {};
}

// The nbt_internal macro above cannot invoke vec directly because it uses
// local_inner_macros. A vec invocation there would resolve to $crate::vec.
// Instead invoke vec here outside of local_inner_macros.
#[macro_export]
#[doc(hidden)]
macro_rules! nbt_internal_vec {
    ($($content:tt)*) => {
        vec![$($content)*]
    };
}
