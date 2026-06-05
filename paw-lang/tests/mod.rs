#[cfg(test)]
mod tests {
    use std::fs::{read_dir, read_to_string};
    use std::path::Path;

    #[test]
    fn compile_cases() {
        let cases_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/cases");
        let cases = read_dir(&cases_dir)
            .unwrap_or_else(
                |_| panic!("could not read {}", cases_dir.display())
            );

        let mut errors = vec![];
        let mut msgs = vec![];

        let mut entries: Vec<_> = cases
            .map(|e| e.unwrap())
            .collect();

        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let path = entry.path();
            let name = path.display().to_string();

            if path.extension().and_then(|e| e.to_str()) != Some("paw") {
                continue;
            }

            match run_case(&path) {
                Ok(_) => msgs.push(format!("  ok  {name}")),
                Err(msg) => {
                    msgs.push(format!("FAIL  {name}"));
                    errors.push(msg);
                }
            }
        }

        println!("\n{} case(s) ran", msgs.len());
        for msg in &msgs {
            println!("{msg}");
        }

        if !errors.is_empty() {
            panic!("\n\nfailed:\n\n{}", errors.join("\n\n---\n\n"));
        }
    }

    fn run_case(path: &Path) -> Result<(), String> {
        let contents = read_to_string(path)
            .map_err(|e| format!("{}: {e}", path.display()))?;

        let (source, expected) = split_case(&contents, path)?;

        let got = paw_lang::compile(&source)
            .map_err(|e| format!("{}: compile error:\n{e}", path.display()))?;

        let got_lines: Vec<&str> = got.lines().collect();
        let exp_lines: Vec<&str> = expected.lines().collect();

        if got_lines.len() != exp_lines.len() {
            return Err(format!(
                "{}: line count mismatch ({} vs {})\n--- got ---\n{}\n--- expected ---\n{}",
                path.display(),
                
                got_lines.len(),
                exp_lines.len(),
                got,
                expected,
            ));
        }

        for (i, (got_line, exp_line)) in got_lines
            .iter()
            .zip(exp_lines.iter())
            .enumerate()
        {
            if got_line.trim() != exp_line.trim() {
                return Err(format!(
                    "{}: line {} mismatch\n  got:\t{got_line:?}\n  expected: {exp_line:?}\n--- full output ---\n{}",
                    path.display(),
                    i + 1,
                    got,
                ));
            }
        }

        Ok(())
    }

    fn split_case(contents: &str, path: &Path) -> Result<(String, String), String> {
        let mut source_lines = vec![];
        let mut expected_lines = vec![];
        let mut in_expected = false;

        for line in contents.lines() {
            if line.starts_with("// ---") {
                if line.starts_with("// --- Expected") {
                    in_expected = true;
                }
                continue;
            }

            if line.starts_with("//") {
                continue;
            }

            if in_expected {
                expected_lines.push(line.to_string());
            } else {
                source_lines.push(line.to_string());
            }
        }

        if !in_expected {
            return Err(format!(
                "{}: missing '// --- Expected' section",
                path.display()
            ));
        }

        Ok((source_lines.join("\n"), expected_lines.join("\n")))
    }
}
