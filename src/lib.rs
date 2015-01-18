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
    let mut attrs = vec!();

    // the raw struct definition
    let def = ast::StructDef {
        fields: vec!(),
        ctor_id: None,
    };

    // set the struct attributes
    // things like "allow", "derive" etc.
    let mut traits = vec!();
    traits.push_all(&*info.derive);

    attrs.push(attribute(cx, "allow", vec!["non_snake_case"]));
    attrs.push(attribute(cx, "derive", traits));

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

    // parse the derive keyword
    //
    // derive => (Show, Copy),
    // ^ parse this word
    parser.parse_ident();

    // derive => (Show, Copy),
    //        ^ parse this
    parser.eat(&token::FatArrow);

    // derive => (Show, Copy),
    //           ^ parse this
    parser.eat(&token::OpenDelim(token::Paren));

    // vec to store the derive traits
    let mut derive = vec!();

    // eat up until the closing paren
    //
    // derive => (Show, Copy)
    //                      ^ parse until here
    while !parser.eat(&token::CloseDelim(token::Paren)) {
        derive.push(parser.parse_ident().as_str().to_string());
        parser.eat(&token::Comma);
    }

    StructInfo {
        name: struct_name,
        derive: derive,
    }
}

struct StructInfo {
    name: ast::Ident,
    derive: Vec<String>,
}

fn attribute<S, T>(cx: &ExtCtxt, name: S, items: Vec<T>) -> ast::Attribute
where S: Str, T: Str {
    let sp = codemap::DUMMY_SP;
    let its = items.into_iter().map(|s| meta_item(cx, s.as_slice())).collect();
    let mi = cx.meta_list(sp, intern(name.as_slice()), its);
    cx.attribute(sp, mi)
}

fn meta_item(cx: &ExtCtxt, s: &str) -> P<ast::MetaItem> {
    cx.meta_word(codemap::DUMMY_SP, intern(s))
}

fn intern(s: &str) -> token::InternedString {
    token::intern_and_get_ident(s)
}
