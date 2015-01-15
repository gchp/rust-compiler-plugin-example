#![feature(plugin_registrar)]
#![allow(unstable)]

extern crate rustc;
extern crate syntax;

use rustc::plugin::Registry;
use syntax::ast;
use syntax::codemap;
use syntax::parse::token;
use syntax::parse::parser::Parser;
use syntax::ext::base::{ExtCtxt, MacResult, MacItems};
use syntax::ext::build::AstBuilder;
use syntax::ptr::P;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_macro("generate_struct", expand);
}

fn expand(cx: &mut ExtCtxt, _sp: codemap::Span, args: &[ast::TokenTree]) -> Box<MacResult + 'static> {
    let mut parser = cx.new_parser_from_tts(args);

    let struct_info = parse_struct_info(&mut parser);

    generate_struct(cx, struct_info)
}

/// Generate a struct with the given StructInfo
///
/// This covers generating both the declaration and fields
fn generate_struct(cx: &mut ExtCtxt, info: StructInfo) -> Box<MacResult + 'static> {
    let mut its = vec!();
    its.push(generate_struct_declaration(cx, info));

    MacItems::new(its.into_iter())
}

/// Generate the struct declaration based on the given StructInfo
///
/// This will give back the full struct declaration including the #[derive]
/// attribute and all fields.
fn generate_struct_declaration(cx: &ExtCtxt, info: StructInfo) -> P<ast::Item> {
    let name = info.name.clone();
    let attrs = vec!();

    // the raw struct definition
    let def = ast::StructDef {
        fields: vec!(),
        ctor_id: None,
    };
    let st = cx.item_struct(codemap::DUMMY_SP, name, def);
    cx.item(codemap::DUMMY_SP, name, attrs, st.node.clone()).map(|mut it| {
        it.vis = ast::Public;
        it
    })
}

fn parse_struct_info(parser: &mut Parser) -> StructInfo {
    // parse the 'name' portion
    //
    // name => Post,
    // ^ parse this
    parser.parse_ident();
    // parse the fat arrow portion
    //
    // name => Post,
    //      ^ parse this
    parser.eat(&token::FatArrow);
    // parse the struct name
    //
    // name => Post,
    //         ^ parse this
    let struct_name = parser.parse_ident();

    // parse up to the comma
    //
    // name => Post,
    //             ^ parse to here
    parser.eat(&token::Comma);

    StructInfo {
        name: struct_name,
    }
}

struct StructInfo {
    name: ast::Ident,
}
