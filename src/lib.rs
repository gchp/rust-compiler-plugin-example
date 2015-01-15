#![feature(plugin_registrar)]
#![allow(unstable)]

extern crate rustc;
extern crate syntax;

use rustc::plugin::Registry;
use syntax::ast;
use syntax::codemap;
use syntax::ext::base::{ExtCtxt, MacResult,  DummyResult};

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
        reg.register_macro("generate_struct", expand);
}

fn expand(cx: &mut ExtCtxt, sp: codemap::Span, args: &[ast::TokenTree]) -> Box<MacResult + 'static> {
    DummyResult::any(sp)
}
