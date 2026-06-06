pub(crate) mod codegen;
pub(crate) mod lexer;
pub(crate) mod core;

use lexer::scanner::Scanner;
use lexer::parser::Parser;
use codegen::{Codegen, emitter::Emitter};
use miette::{NamedSource, Report, Diagnostic};
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
#[error("compilation failed")]
struct SourcedErrors<E: Diagnostic + Send + Sync + 'static> {
    #[source_code]
    src: NamedSource<String>,
    #[related]
    errors: Vec<E>,
}

pub fn compile_report(src: &str, file_name: &str) -> Result<String, Report> {
    let named = NamedSource::new(file_name, src.to_string());

    let tokens = Scanner::new(src)
        .scan_tokens()
        .map_err(|e| {
            Report::new(SourcedErrors {
                src: named.clone(),
                errors: e.errors
            })
        })?;

    let stmts = Parser::new(tokens)
        .parse()
        .map_err(|e| {
            Report::new(SourcedErrors {
                src: named,
                errors: e.errors
            })
        })?;

    let mut buf = Vec::new();
    let emitter = Emitter::new(&mut buf);
    let mut cg = Codegen::new(emitter);

    cg.emit_program(&stmts);
    cg.finish().map_err(|e| Report::msg(e.to_string()))?;

    Ok(String::from_utf8(buf).expect("emitter produced invalid utf-8"))
}

pub fn compile(src: &str) -> Result<String, String> {
    let tokens = Scanner::new(src).scan_tokens()
        .map_err(
            |e| e.errors
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("\n")
            )?;

    let stmts = Parser::new(tokens).parse()
        .map_err(
            |e| e.errors
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("\n")
            )?;

    let mut buf = Vec::new();
    let emitter = Emitter::new(&mut buf);
    let mut cg = Codegen::new(emitter);
    
    cg.emit_program(&stmts);
    cg.finish().map_err(|e| e.to_string())?;

    Ok(String::from_utf8(buf).expect("emitter produced invalid utf-8"))
}