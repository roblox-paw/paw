set quiet

set shell := ["bash", "-cu"]
set windows-shell := ["powershell.exe", "-c"]

alias t  := test
alias ut := unit
alias cs := cases

# run all tests (unit + cases)
[group("test")]
[default]
test:
    cargo test

# run only unit tests (src/ inline tests)
[group("test")]
unit:
    cargo test --lib

# run only case files (tests/cases/*.paw)
[group("test")]
[no-exit-message]
cases:
    cargo test compile_cases -- --nocapture

# run a specific case by name (e.g. just case if_statement)
[group("test")]
[no-exit-message]
case name:
    cargo test compile_cases -- --nocapture 2>&1 | grep -E "(ok|FAIL).*{{name}}"