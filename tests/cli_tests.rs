use assert_cmd::Command;
use std::fs;

#[test]
fn test_basic_flow() {
    // This test covers the basic flow of processing a CSV file with valid records. It checks that the output matches the expected results.
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).expect("Failed to find binary");
    let input_path = "fixtures/basic_flow.csv";
    let expected_output_path = "fixtures/basic_flow_expected.csv";

    let expected_string =
        fs::read_to_string(expected_output_path).expect("Failed to read expected output file");

    cmd.arg(input_path);
    cmd.assert()
        .success()
        .stdout(predicates::str::diff(expected_string));
}

#[test]
fn test_missing_input_file() {
    // If the input file is missing, there is no data to process. So, we should exit immediately with an error.
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).expect("Failed to find binary");
    let input_path = "fixtures/non_existent_file.csv";

    cmd.arg(input_path);
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("No such file or directory"));
}

#[test]
fn test_invalid_csv_header_format() {
    // If the headers are invalid, then the file can be considered malformed, the program should print an error message and exit without processing any records.
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).expect("Failed to find binary");
    let input_path = "fixtures/invalid_header_format.csv";

    cmd.arg(input_path);
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("Error parsing CSV record"))
        .stdout(predicates::str::is_empty());
}

#[test]
fn test_invalid_csv_record_format() {
    // If a record is invalid, then the file can be considered malformed, the program should print an error message and exit without processing any more records.
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).expect("Failed to find binary");
    let input_path = "fixtures/invalid_record_format.csv";

    cmd.arg(input_path);
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("Error parsing CSV record"))
        .stdout(predicates::str::is_empty());
}
