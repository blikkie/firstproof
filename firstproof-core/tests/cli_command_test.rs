use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn test_cli_command_single_shot_argument() {
    let cli_path = env!("CARGO_BIN_EXE_firstproof-cli");
    let cmd_json = r#"{"command":"set_exposure","ev":0.75}"#;

    let output = Command::new(cli_path)
        .args(["command", cmd_json])
        .output()
        .expect("Failed to execute firstproof-cli command");

    assert!(output.status.success(), "CLI command failed: {:?}", output);
    let stdout = String::from_utf8(output.stdout).expect("valid utf8");
    let res: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON response");

    assert_eq!(res["status"], "ok");
    assert!((res["exposure"].as_f64().unwrap() - 0.75).abs() < 1e-4);
    assert!((res["settings"]["exposure"].as_f64().unwrap() - 0.75).abs() < 1e-4);
}

#[test]
fn test_cli_command_piped_stdin_stream() {
    let cli_path = env!("CARGO_BIN_EXE_firstproof-cli");

    let mut child = Command::new(cli_path)
        .arg("command")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn firstproof-cli command");

    let stdin_payload = concat!(
        r#"{"command":"apply_edit_delta","delta":{"contrast":15.0}}"#, "\n",
        r#"{"command":"set_temperature","kelvin":6200.0}"#, "\n",
        r#"{"command":"set_tint","tint":-8.0}"#, "\n",
    );

    {
        let mut stdin = child.stdin.take().expect("Failed to open stdin");
        stdin.write_all(stdin_payload.as_bytes()).expect("write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to wait on child");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("valid utf8");
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 3, "Expected 3 response lines for 3 input commands, got: {:?}", lines);

    let res1: serde_json::Value = serde_json::from_str(lines[0]).expect("parse line 1");
    assert_eq!(res1["status"], "ok");
    assert!((res1["settings"]["contrast"].as_f64().unwrap() - 15.0).abs() < 1e-4);

    let res2: serde_json::Value = serde_json::from_str(lines[1]).expect("parse line 2");
    assert_eq!(res2["status"], "ok");
    assert!((res2["temperature"].as_f64().unwrap() - 6200.0).abs() < 1e-4);

    let res3: serde_json::Value = serde_json::from_str(lines[2]).expect("parse line 3");
    assert_eq!(res3["status"], "ok");
    assert!((res3["tint"].as_f64().unwrap() - (-8.0)).abs() < 1e-4);
}

#[test]
fn test_cli_command_json_rpc_envelope_compatibility() {
    let cli_path = env!("CARGO_BIN_EXE_firstproof-cli");
    let rpc_json = r#"{"method":"apply_edit_delta","params":{"exposure":1.25,"vibrance":25.0}}"#;

    let output = Command::new(cli_path)
        .args(["command", rpc_json])
        .output()
        .expect("Failed to execute firstproof-cli command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("valid utf8");
    let res: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON response");

    assert_eq!(res["status"], "ok");
    assert!((res["settings"]["exposure"].as_f64().unwrap() - 1.25).abs() < 1e-4);
    assert!((res["settings"]["vibrance"].as_f64().unwrap() - 25.0).abs() < 1e-4);
}

#[test]
fn test_cli_command_invalid_json_handling() {
    let cli_path = env!("CARGO_BIN_EXE_firstproof-cli");
    let invalid_json = "not a json string";

    let output = Command::new(cli_path)
        .args(["command", invalid_json])
        .output()
        .expect("Failed to execute firstproof-cli command");

    let stdout = String::from_utf8(output.stdout).expect("valid utf8");
    let res: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON response");

    assert_eq!(res["status"], "error");
    assert!(res["message"].as_str().unwrap().contains("Invalid JSON"));
}
