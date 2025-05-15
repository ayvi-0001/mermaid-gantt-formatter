use assert_fs::{assert::PathAssert, fixture::{FileTouch, FileWriteFile, PathChild, PathCopy}};
use indoc::indoc;

#[test]
fn invalid_diagram_from_stdin() {
    let mut cmd = assert_cmd::Command::cargo_bin(clap::crate_name!()).unwrap();
    let assert = cmd
        .write_stdin("invalid text for mermaid diagram")
        .assert();
    assert
        .failure()
        .code(2)
        .stdout("")
        .stderr("error: Invalid diagram\n");
}

#[test]
fn empty_stdin_buffer() {
    let mut cmd = assert_cmd::Command::cargo_bin(clap::crate_name!()).unwrap();
    let assert = cmd.write_stdin("").assert();
    assert
        .failure()
        .code(2)
        .stdout("")
        .stderr("error: No input detected.\n");
}

#[test]
fn input_file_not_found() {
    let mut cmd = assert_cmd::Command::cargo_bin(clap::crate_name!()).unwrap();
    let assert = cmd.args(["-i", "non-existing-file.mmd"]).assert();
    assert
        .failure()
        .code(2)
        .stdout("")
        .stderr("error: The system cannot find the file specified. (os error 2)\n");
}

#[test]
fn stdout_from_input_file() {
    let mut cmd = assert_cmd::Command::cargo_bin(clap::crate_name!()).unwrap();
    let assert = cmd.args(["-i", "gantt_chart.mmd"]).assert();
    assert.code(0).stdout(EXPECTED_OUTPUT).stderr("");
}

#[test]
fn output_file_from_input_file() {
    let temp = assert_fs::TempDir::new().unwrap();
    let output_file = temp.child("output.mmd");

    output_file.touch().unwrap();

    let mut cmd = assert_cmd::Command::cargo_bin(clap::crate_name!()).unwrap();
    let assert = cmd
        .args([
            "-i",
            "gantt_chart.mmd",
            "-o",
            output_file.to_str().unwrap(),
        ])
        .assert();

    output_file.assert(EXPECTED_OUTPUT);
    assert.code(0).stdout("").stderr("");
}

#[test]
fn input_file_inplace() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.copy_from(".", &["gantt_chart.mmd"]).ok();

    let input_file = temp.child("gantt_chart.mmd");
    input_file
        .write_file(std::path::Path::new("gantt_chart.mmd"))
        .ok();

    let mut cmd = assert_cmd::Command::cargo_bin(clap::crate_name!()).unwrap();
    let assert = cmd
        .args(["-i", input_file.path().to_str().unwrap(), "-I"])
        .assert();

    input_file.assert(EXPECTED_OUTPUT);
    assert.code(0).stdout("").stderr("");
}

#[test]
fn stdout_from_piped_stdin() {
    let mut cmd = assert_cmd::Command::cargo_bin(clap::crate_name!()).unwrap();
    let assert = cmd
        .pipe_stdin("gantt_chart.mmd")
        .expect("")
        .assert();
    assert.code(0).stdout(EXPECTED_OUTPUT).stderr("");
}

const EXPECTED_OUTPUT: &str = indoc! {"\
    gantt
      dateFormat YYYY-MM-DD
      title Adding GANTT diagram functionality to mermaid
      %% (`excludes` accepts specific dates in YYYY-MM-DD format, days of the week (\"sunday\") or \"weekends\", but not the word \"weekdays\".)
      excludes weekends
      %% Weekend (v\\11.0.0+)
      weekend friday

    %% Do first
      section A section
        Completed task                      : done  ,                  des1   , 2014-01-06, 2014-01-08
        Active task                         : active,                  des2   , 2014-01-09, 3d
        Future task                         :                          des3   , after des2, 5d
        Future task2                        :                          des4   , after des3, 5d

      section Critical tasks
        Completed task in the critical line : done  , crit,                     2014-01-06, 24h
        Implement parser and jison          : done  , crit,                     after des1, 2d
        Create tests for parser             : active, crit,                                 3d
        Future task in critical line        :         crit,                                 5d
    %%  Create tests for renderer           :                                               2d
        Add to mermaid                      :                                               until isadded
        Functionality added                 :               milestone, isadded, 2014-01-25, 0d

      section Documentation
        Describe gantt syntax               : active,                  a1     , after des1, 3d
        Add gantt diagram to demo page      :                                   after a1  , 20h
        Add another diagram to demo page    :                          doc1   , after a1  , 48h

      section Last section
    %%  Refer to section Documentation
        Describe gantt syntax               :                                   after doc1, 3d
        Add gantt diagram to demo page      :                                               20h
        Add another diagram to demo page    :                                               48h
    "
};
