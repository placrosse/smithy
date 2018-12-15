use crate::types::{
  AttributeOrEventHandler,
  SplitByType,
  TokenStreamEventHandlingInfoPair,
  TokenTreeSlice,
  UIEventHandlingInfo,
};
use nom::{
  alt,
  apply,
  call,
  delimited,
  error_position,
  map,
  named,
  tuple,
  tuple_parser,
};
use proc_macro2::{
  Delimiter,
  Spacing,
  TokenStream,
};
use quote::quote;
use std::iter::Extend;

use super::{
  make_smithy_tokens::{
    make_html_tokens,
    make_text_node,
  },
  util,
  window_event_handlers::match_window_event_handlers,
};

named!(
  match_self_closing_token <TokenTreeSlice, TokenStreamEventHandlingInfoPair>,
  map!(
    delimited!(
      apply!(util::match_punct, Some('<'), Some(Spacing::Alone), vec![]),
      tuple!(
        apply!(util::match_ident, None, false),
        many_0_custom!(super::attributes::match_attribute)
      ),
      tuple!(
        apply!(util::match_punct, Some('/'), Some(Spacing::Joint), vec![]),
        apply!(util::match_punct, Some('>'), None, vec![])
      )
    ),
    |(name, attributes_and_event_handlers)| {
      let (attributes, event_handlers) = attributes_and_event_handlers.split_by_type();
      let component = make_html_tokens(name, attributes, vec![]);
      let event_handlers = event_handlers
        .into_iter()
        .map(UIEventHandlingInfo::from_string_token_stream_pair)
        .collect();
      (component, event_handlers)
    }
  )
);

named!(
  match_opening_tag <TokenTreeSlice, (String, Vec<AttributeOrEventHandler>)>,
  delimited!(
    apply!(util::match_punct, Some('<'), Some(Spacing::Alone), vec![]),
    tuple!(
      apply!(util::match_ident, None, false),
      many_0_custom!(super::attributes::match_attribute)
    ),
    apply!(util::match_punct, Some('>'), None, vec![])
  )
);

named!(
  match_closing_tag <TokenTreeSlice, String>,
  delimited!(
    tuple!(
      apply!(util::match_punct, Some('<'), Some(Spacing::Joint), vec![]),
      apply!(util::match_punct, Some('/'), Some(Spacing::Alone), vec![])
    ),
    apply!(util::match_ident, None, false),
    apply!(util::match_punct, Some('>'), None, vec![])
  )
);

named!(
  match_regular_token <TokenTreeSlice, TokenStreamEventHandlingInfoPair>,
  map!(
    tuple!(
      match_opening_tag,
      many_0_custom!(match_node),
      match_closing_tag
    ),
    |((name, attributes_and_event_handlers), children_and_events, closing_tag_name)| {
      // TODO add a descriptive error message
      assert_eq!(name, closing_tag_name);
      let (attributes, event_handlers) = attributes_and_event_handlers.split_by_type();
      let (children, child_event_infos) = children_and_events.split_by_type();
      let token_stream = make_html_tokens(name, attributes, children);
      let mut event_infos: Vec<UIEventHandlingInfo> = event_handlers
        .into_iter()
        .map(UIEventHandlingInfo::from_string_token_stream_pair)
        .collect();
      event_infos.extend(child_event_infos.into_iter());
      (token_stream, event_infos)
    }
  )
);

named!(
  match_html_token <TokenTreeSlice, TokenStreamEventHandlingInfoPair>,
  alt!(
    match_self_closing_token
      | match_regular_token
  )
);

// N.B. this is separated because there seems to be a bug
// in many_1_custom. TODO look at this
named!(
  match_ident_2 <TokenTreeSlice, String>,
  alt!(
    apply!(util::match_ident, None, true)
      | apply!(util::match_punct, None, None, vec!['<'])
      | call!(util::match_literal_as_string)
  )
);

named!(
  match_string_as_node <TokenTreeSlice, TokenStreamEventHandlingInfoPair>,
  map!(
    many_1_custom!(match_ident_2),
    |vec| {
      let joined = vec.iter().map(|ident| ident.to_string()).collect::<Vec<String>>().join("");
      (make_text_node(joined), vec![])
    }
  )
);

named!(
  match_group <TokenTreeSlice, TokenStreamEventHandlingInfoPair>,
  map!(
    apply!(util::match_group, Some(Delimiter::Brace)),
    |x| (quote!(#x.render()), vec![
      UIEventHandlingInfo {
        reversed_path: vec![],
        event: None,
        callback: quote!(#x),
        is_group: true,
      }
    ])
  )
);

named!(
  match_node <TokenTreeSlice, TokenStreamEventHandlingInfoPair>,
  alt!(
    match_html_token
      | match_string_as_node
      | match_group
  )
);

named!(
  pub match_html_component <TokenTreeSlice, TokenStream>,
  map!(
    tuple!(
      many_0_custom!(match_window_event_handlers),
      many_1_custom!(match_node)
    ),
    |(window_event_handler_infos, dom_vec)| {
      let (vec_of_node_tokens, event_handling_infos) = dom_vec.into_iter().enumerate()
        .fold(
          (vec![], vec![]),
          |(mut vec_of_node_tokens, mut event_handling_infos), (i, (token, current_event_handling_infos))| {
            vec_of_node_tokens.push(token);

            let mut vec = current_event_handling_infos.into_iter().map(|mut info| {
              info.reversed_path.push(i);
              info
            }).collect::<Vec<UIEventHandlingInfo>>();
            event_handling_infos.append(&mut vec);
            (vec_of_node_tokens, event_handling_infos)
          }
        );
      let token = util::reduce_vec_to_node(&vec_of_node_tokens);
      super::make_smithy_tokens::make_component(token, event_handling_infos, window_event_handler_infos)
    }
  )
);
