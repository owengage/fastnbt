// Taken from serde_json and modified for NBT
// https://github.com/serde-rs/json/blob/829175e6069fb16672875f125f6afdd7c6da1dec/src/macros.rs#L60-L303
#[macro_export]
macro_rules! nbt {
    // Done with trailing comma.
    (@array [$($elems:expr,)*]) => {
        vec![$($elems,)*]
    };

    // Done without trailing comma.
    (@array [$($elems:expr),*]) => {
        vec![$($elems),*]
    };

    // Next element is an array.
    (@array [$($elems:expr,)*] [$($array:tt)*] $($rest:tt)*) => {
        nbt!(@array [$($elems,)* nbt!([$($array)*])] $($rest)*)
    };

    // Next element is a map.
    (@array [$($elems:expr,)*] {$($map:tt)*} $($rest:tt)*) => {
        nbt!(@array [$($elems,)* nbt!({$($map)*})] $($rest)*)
    };

    // Next element is an expression followed by comma.
    (@array [$($elems:expr,)*] $next:expr, $($rest:tt)*) => {
        nbt!(@array [$($elems,)* nbt!($next),] $($rest)*)
    };

    // Last element is an expression with no trailing comma.
    (@array [$($elems:expr,)*] $last:expr) => {
        nbt!(@array [$($elems,)* nbt!($last)])
    };

    // Comma after the most recent element.
    (@array [$($elems:expr),*] , $($rest:tt)*) => {
        nbt!(@array [$($elems,)*] $($rest)*)
    };

    // Unexpected token after most recent element.
    (@array [$($elems:expr),*] $unexpected:tt $($rest:tt)*) => {
        nbt_unexpected!($unexpected)
    };

    /* ------------ IntArray types ------------ */

    // Done with trailing comma.
    (@int_array [$($elems:expr,)*]) => {
        vec![$($elems,)*]
    };

    // Done without trailing comma.
    (@int_array [$($elems:expr),*]) => {
        vec![$($elems),*]
    };

    // Next element is an expression followed by comma.
    (@int_array [$($elems:expr,)*] $next:expr, $($rest:tt)*) => {
        nbt!(@int_array [$($elems,)* $next,] $($rest)*)
    };

    // Last element is an expression with no trailing comma.
    (@int_array [$($elems:expr,)*] $last:expr) => {
        nbt!(@int_array [$($elems,)* $last])
    };

    // Comma after the most recent element.
    (@int_array [$($elems:expr),*] , $($rest:tt)*) => {
        nbt!(@int_array [$($elems,)*] $($rest)*)
    };

    // Unexpected token after most recent element.
    (@int_array [$($elems:expr),*] $unexpected:tt $($rest:tt)*) => {
        nbt_unexpected!($unexpected)
    };

    //////////////////////////////////////////////////////////////////////////
    // TT muncher for parsing the inside of an object {...}. Each entry is
    // inserted into the given map variable.
    //
    // Must be invoked as: nbt!(@object $map () ($($tt)*) ($($tt)*))
    //
    // We require two copies of the input tokens so that we can match on one
    // copy and trigger errors on the other copy.
    //////////////////////////////////////////////////////////////////////////

    // Done.
    (@object $object:ident () () ()) => {};

    // Insert the current entry followed by trailing comma.
    (@object $object:ident [$($key:tt)+] ($value:expr) , $($rest:tt)*) => {
        let _ = $object.insert(($($key)+).into(), $value);
        nbt!(@object $object () ($($rest)*) ($($rest)*));
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
        nbt!(@object $object [$($key)+] (nbt!([$($array)*])) $($rest)*);
    };

    // Next value is a map.
    (@object $object:ident ($($key:tt)+) (: {$($map:tt)*} $($rest:tt)*) $copy:tt) => {
        nbt!(@object $object [$($key)+] (nbt!({$($map)*})) $($rest)*);
    };

    // Next value is an expression followed by comma.
    (@object $object:ident ($($key:tt)+) (: $value:expr , $($rest:tt)*) $copy:tt) => {
        nbt!(@object $object [$($key)+] (nbt!($value)) , $($rest)*);
    };

    // Last value is an expression with no trailing comma.
    (@object $object:ident ($($key:tt)+) (: $value:expr) $copy:tt) => {
        nbt!(@object $object [$($key)+] (nbt!($value)));
    };

    // Missing value for last entry. Trigger a reasonable error message.
    (@object $object:ident ($($key:tt)+) (:) $copy:tt) => {
        // "unexpected end of macro invocation"
        nbt!();
    };

    // Missing colon and value for last entry. Trigger a reasonable error
    // message.
    (@object $object:ident ($($key:tt)+) () $copy:tt) => {
        // "unexpected end of macro invocation"
        nbt!();
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
        nbt!(@object $object ($key) (: $($rest)*) (: $($rest)*));
    };

    // Refuse to absorb colon token into key expression.
    (@object $object:ident ($($key:tt)*) (: $($unexpected:tt)+) $copy:tt) => {
        nbt_expect_expr_comma!($($unexpected)+);
    };

    // Munch a token into the current key.
    (@object $object:ident ($($key:tt)*) ($tt:tt $($rest:tt)*) $copy:tt) => {
        nbt!(@object $object ($($key)* $tt) ($($rest)*) ($($rest)*));
    };

    //////////////////////////////////////////////////////////////////////////
    // The main implementation.
    //
    // Must be invoked as: nbt!($($json)+)
    //////////////////////////////////////////////////////////////////////////

    ([B;]) => {
        $crate::Value::ByteArray($crate::ByteArray::new(vec![]))
    };

    ([I;]) => {
        $crate::Value::IntArray($crate::IntArray::new(vec![]))
    };

    ([L;]) => {
        $crate::Value::LongArray($crate::LongArray::new(vec![]))
    };

    ([]) => {
        $crate::Value::List(vec![])
    };

    ([B; $($tt:tt)+ ]) => {
        $crate::Value::ByteArray($crate::ByteArray::new(nbt!(@int_array [] $($tt)+)))
    };

    ([I; $($tt:tt)+ ]) => {
        $crate::Value::IntArray($crate::IntArray::new(nbt!(@int_array [] $($tt)+)))
    };

    ([L; $($tt:tt)+ ]) => {
        $crate::Value::LongArray($crate::LongArray::new(nbt!(@int_array [] $($tt)+)))
    };

    ([ $($tt:tt)+ ]) => {
        $crate::Value::List(nbt!(@array [] $($tt)+))
    };

    ({}) => {
        $crate::Value::Compound(std::collections::HashMap::new())
    };

    ({ $($tt:tt)+ }) => {
        $crate::Value::Compound({
            let mut object = std::collections::HashMap::new();
            nbt!(@object object () ($($tt)+) ($($tt)+));
            object
        })
    };

    // Any value of T where fastnbt::Value: From<T>
    ($other:expr) => {
        $crate::to_value($other)
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
