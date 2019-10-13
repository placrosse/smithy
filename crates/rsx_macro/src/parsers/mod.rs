use crate::types::*;

fn parse_macro_item(input: proc_macro2::TokenStream) -> TokenStreamIResult<TokenStream> {
  let (rest, group_contents) =
    crate::utils::parse_group_with_delimiter(input, Some(proc_macro2::Delimiter::Parenthesis))?;
  Ok((rest, group_contents))
}

fn parse_literal(input: proc_macro2::TokenStream) -> TokenStreamIResult<TokenStream> {
  let (rest, group_contents) =
    crate::utils::parse_group_with_delimiter(input, Some(proc_macro2::Delimiter::Parenthesis))?;
  Ok((rest, group_contents))
}

pub fn parse_items(input: proc_macro2::TokenStream) -> TokenStreamIResult<Vec<TokenStream>> {
  let (rest, parsed) =
    crate::utils::many_0(nom::branch::alt((parse_macro_item, parse_literal)))(input)?;
  let (rest, _) = crate::utils::ensure_consumed(rest)?;
  Ok((rest, parsed))
}
