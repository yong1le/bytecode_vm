// Tests (26 suites)
// bool
// string
// comments
// print
// operator
// logical_operator
// variable
// assignment
// block
// if
// while
// for

// function
// call
// return
// closure
// class
// field
// constructor
// method
// this
// inheritance
// super
// regression
// limit
// benchmark

use lox_bytecode_vm::interpret;
use lox_bytecode_vm::vm::VM;
use std::fs;
use std::io::{self};
use std::path::{Path, PathBuf};

// Define test suites - for each directory in tests/lox
#[test]
fn test_bool() {
    run_test_suite("bool");
}

#[test]
fn test_string() {
    run_test_suite("string");
}

#[test]
fn test_comments() {
    run_test_suite("comments");
}

#[test]
fn test_print() {
    run_test_suite("print");
}

#[test]
fn test_operator() {
    run_test_suite("operator");
}

#[test]
fn test_logical_operator() {
    run_test_suite("logical_operator");
}

#[test]
fn test_variable() {
    run_test_suite("variable");
}

#[test]
fn test_assignment() {
    run_test_suite("assignment");
}

#[test]
fn test_block() {
    run_test_suite("block");
}

#[test]
fn test_if() {
    run_test_suite("if");
}

#[test]
fn test_while() {
    run_test_suite("while");
}

#[test]
fn test_for() {
    run_test_suite("for");
}

#[test]
#[ignore]
fn test_function() {
    run_test_suite("function");
}

#[test]
#[ignore]
fn test_call() {
    run_test_suite("call");
}

#[test]
#[ignore]
fn test_return() {
    run_test_suite("return");
}

#[test]
#[ignore]
fn test_closure() {
    run_test_suite("closure");
}

#[test]
#[ignore]
fn test_class() {
    run_test_suite("class");
}

#[test]
#[ignore]
fn test_field() {
    run_test_suite("field");
}

#[test]
#[ignore]
fn test_constructor() {
    run_test_suite("constructor");
}

#[test]
#[ignore]
fn test_method() {
    run_test_suite("method");
}

#[test]
#[ignore]
fn test_this() {
    run_test_suite("this");
}

#[test]
#[ignore]
fn test_inheritance() {
    run_test_suite("inheritance");
}

#[test]
#[ignore]
fn test_super() {
    run_test_suite("super");
}

#[test]
#[ignore]
fn test_regression() {
    run_test_suite("regression");
}

#[test]
#[ignore]
fn test_limit() {
    run_test_suite("limit");
}

#[test]
#[ignore]
fn test_benchmark() {
    run_test_suite("benchmark");
}

// Function to capture stdout and stderr during interpret execution
fn capture_output_from_interpret(source: &str) -> io::Result<String> {
    // Create buffers to capture stdout and stderr
    let mut stdout_buffer = Vec::new();
    let mut stderr_buffer = Vec::new();

    // Create a VM instance
    let mut vm = VM::new(Box::new(&mut stdout_buffer));
    // Run interpret (which will print to our redirected stdout/stderr)

    interpret(source, &mut vm, &mut stderr_buffer);

    drop(vm);

    // Get the captured output
    let stdout_output = String::from_utf8_lossy(&stdout_buffer);
    let stderr_output = String::from_utf8_lossy(&stderr_buffer);

    // Combine stdout and stderr
    let mut combined_output = String::new();
    if !stdout_output.is_empty() {
        combined_output.push_str(&stdout_output);
    }
    if !stderr_output.is_empty() {
        // If we have both stdout and stderr, add a separator
        if !combined_output.is_empty() {
            combined_output.push('\n');
        }
        combined_output.push_str(&stderr_output);
    }

    Ok(combined_output)
}

// Function to get expected output - tries .expected file first, then falls back to comments
fn get_expected_output(test_path: &Path) -> io::Result<String> {
    // Try to read from .expected file first
    let expected_path = test_path.with_extension("expected");
    match fs::read_to_string(&expected_path) {
        Ok(content) => Ok(content.trim().to_string()),
        Err(e) => Err(e),
    }
}

// Helper function to run a test suite
fn run_test_suite(suite_name: &str) {
    let suite_path = PathBuf::from("tests/lox").join(suite_name);

    // Get and sort test files
    let test_files = fs::read_dir(&suite_path)
        .unwrap_or_else(|_| panic!("Failed to read test suite directory: {}", suite_name))
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "lox"))
        .map(|entry| entry.path())
        .collect::<Vec<_>>();

    assert!(
        !test_files.is_empty(),
        "No test files found in suite: {}",
        suite_name
    );

    let expected = test_files.len();
    let mut passed = 0;
    let mut failed = 0;

    for test_path in test_files {
        let test_name = test_path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // Read source
        let source = fs::read_to_string(&test_path)
            .unwrap_or_else(|e| panic!("Error reading test file {}: {}", test_path.display(), e));

        // Get expected output
        let expected = get_expected_output(&test_path).unwrap_or_else(|e| {
            panic!(
                "Error getting expected output for {}: {}",
                test_path.display(),
                e
            )
        });

        // Run test and capture output
        let actual = capture_output_from_interpret(&source)
            .unwrap_or_else(|e| panic!("Error capturing output: {}", e))
            .trim()
            .to_string();

        if actual == expected {
            passed += 1;
        } else {
            failed += 1;
            eprintln!(
                "\n=== Test '{}' in suite '{}' failed! ===\nExpected:\n{}\nActual:\n{}\n",
                test_name, suite_name, expected, actual
            )
        }
    }

    assert!(
        expected == passed && failed == 0,
        "\n=== Test suite '{}' finished: {} passed and {} failed. ===\n",
        suite_name,
        passed,
        failed
    )
}
