use assert_cmd::Command;
use std::fs;

#[test]
fn test_basic_flow() {
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
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).expect("Failed to find binary");
    let input_path = "fixtures/non_existent_file.csv";

    cmd.arg(input_path);
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("No such file or directory"));
}

#[test]
fn test_invalid_csv_format() {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).expect("Failed to find binary");
    let input_path = "fixtures/invalid_format.csv";

    cmd.arg(input_path);
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("Error parsing CSV record"));
}
