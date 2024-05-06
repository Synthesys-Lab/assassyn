use proc_macro2::Span;
use syn::punctuated::Punctuated;

use crate::ast::node::WeakSpanned;

pub(crate) fn punctuated_span<T: WeakSpanned, P>(x: &Punctuated<T, P>) -> Option<Span> {
  if let Some(first) = x.first() {
    let first_span = first.span();
    if let Some(last) = x.last() {
      let last_span = last.span();
      return first_span.join(last_span);
    }
  }
  None
}
