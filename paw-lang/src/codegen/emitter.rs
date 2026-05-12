#![allow(dead_code)]
use std::io::{self, Write};

pub struct Emitter<'e> {
    out: &'e mut dyn Write,
    indent: usize,
    error: Option<io::Error>,
}

impl<'e> Emitter<'e> {
    pub fn new(out: &'e mut dyn Write) -> Self {
        Self {
            out,
            indent: 0,
            error: None
        }
    }

    pub fn finish(self) -> io::Result<()> {
        match self.error {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }

    fn write_raw(&mut self, bytes: &[u8]) {
        if self.error.is_none() {
            if let Err(e) = self.out.write_all(bytes) {
                self.error = Some(e);
            }
        }
    }

    pub fn write(&mut self, s: &str) {
        self.write_raw(s.as_bytes());
    }

    pub fn write_spaced(&mut self, s: &str) {
        self.write_raw(b" ");
        self.write_raw(s.as_bytes());
        self.write_raw(b" ");
    }

    pub fn writeln(&mut self, s: &str) {
        self.emit_indent();
        self.write_raw(s.as_bytes());
        self.write_raw(b"\n");
    }

    pub fn newline(&mut self) {
        self.write_raw(b"\n");
    }

    pub fn emit_indent(&mut self) {
        for _ in 0..self.indent {
            self.write_raw(b"\t");
        }
    }

    pub fn indent(&mut self) {
        self.indent += 1;
    }

    pub fn dedent(&mut self) {
        self.indent = self.indent.saturating_sub(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn collect(f: impl FnOnce(&mut Emitter)) -> String {
        let mut buf = Vec::new();
        let mut e = Emitter::new(&mut buf);
        f(&mut e);

        e.finish().expect("unexpected error in test");
        String::from_utf8(buf).expect("emitter produced invalid utf-8")
    }

    #[test]
    fn writeln_no_indent() {
        let out = collect(|e| e.writeln("local x = 1"));
        assert_eq!(out, "local x = 1\n");
    }

    #[test]
    fn writeln_one_indent() {
        let out = collect(|e| {
            e.indent();
            e.writeln("local x = 1");
        });
        assert_eq!(out, "\tlocal x = 1\n");
    }

    #[test]
    fn writeln_two_indent() {
        let out = collect(|e| {
            e.indent();
            e.indent();
            e.writeln("local x = 1");
        });
        assert_eq!(out, "\t\tlocal x = 1\n");
    }

    #[test]
    fn dedent_no_underflow() {
        let out = collect(|e| {
            e.dedent();
            e.dedent();
            e.writeln("x");
        });
        assert_eq!(out, "x\n");
    }

    #[test]
    fn indent_dedent_balance() {
        let out = collect(|e| {
            e.writeln("if true then");
            e.indent();
            e.writeln("local x = 1");
            e.dedent();
            e.writeln("end");
        });
        assert_eq!(out, "if true then\n\tlocal x = 1\nend\n");
    }

    #[test]
    fn continues_line() {
        let out = collect(|e| {
            e.emit_indent();
            e.write("local x");
            e.write(" = ");
            e.write("1");
            e.newline();
        });
        assert_eq!(out, "local x = 1\n");
    }

    #[test]
    fn newline_no_indent() {
        let out = collect(|e| {
            e.indent();
            e.writeln("local x = 1");
            e.newline();
            e.writeln("local y = 2");
        });
        assert_eq!(out, "\tlocal x = 1\n\n\tlocal y = 2\n");
    }
}