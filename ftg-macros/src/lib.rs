mod expand;

use proc_macro::TokenStream;

/**
 * Generates associated c-style structs for c-style struct enum variants.
 *
 * Example:
 * ```
 * #[expand]
 * #[derive(Debug)]
 * enum Enum {
 *     Foo { foo: u8 }
 *     Bar { bar: u8 }
 * }
 * ```
 * generates
 * ```
 * #[derive(Debug)]
 * Foo {
 *     foo: u8
 * }
 *
 * #[derive(Debug)]
 * Bar {
 *     foo: u8
 * }
 *
 * enum Enum {
 *     Foo(Foo)
 *     Bar(Bar)
 * }
 * ```
 */
#[proc_macro_attribute]
pub fn expand(_arg_tokens: TokenStream, item_tokens: TokenStream) -> TokenStream {
    crate::expand::expand(item_tokens)
}
