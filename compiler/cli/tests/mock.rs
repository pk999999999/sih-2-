use std::process::Command;

#[test]
fn mock_cli_runs_complete_pipeline_and_returns_six_indicators() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/mock_investigation.jky");
    let output = Command::new(env!("CARGO_BIN_EXE_jocky-cli"))
        .arg("run").arg(path).arg("--mock").output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["mock"], true);
    assert_eq!(value["reports"][0]["evidence"]["sys"]["hostname"], "FORENSIC-PC-001");
    assert_eq!(value["findings"].as_array().unwrap().len(), 6);
}
