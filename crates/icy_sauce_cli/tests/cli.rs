use icy_sauce::{
    SauceDataType, SauceDate, SauceRecord, SauceRecordBuilder, StripMode, strip_sauce,
};
use serde_json::json;
use std::{fs, path::PathBuf, process::Command};
use tempfile::TempDir;

const PAYLOAD: &[u8] = b"artwork\x00\x82\x1a";

struct Fixture {
    dir: TempDir,
    file: PathBuf,
}

impl Fixture {
    fn new(record: Option<&SauceRecord>) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("art.ans");
        let mut bytes = PAYLOAD.to_vec();
        if let Some(record) = record {
            bytes.extend(record.to_bytes().unwrap());
        }
        fs::write(&file, bytes).unwrap();
        Self { dir, file }
    }

    fn command(&self, operation: &str) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_sauce"));
        command.arg(operation).arg(&self.file);
        command
    }

    fn run(&self, operation: &str, args: &[&str]) {
        let output = self.command(operation).args(args).output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn record(&self) -> SauceRecord {
        let bytes = fs::read(&self.file).unwrap();
        assert_eq!(strip_sauce(&bytes, StripMode::LastStripFinalEof), PAYLOAD);
        SauceRecord::from_bytes(&bytes).unwrap().unwrap()
    }

    fn json_path(&self, value: serde_json::Value) -> PathBuf {
        let path = self.dir.path().join("metadata.json");
        fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
        path
    }
}

fn unusual_record(data_type: SauceDataType) -> SauceRecord {
    SauceRecordBuilder::default()
        .title(vec![0x82; 35].into())
        .unwrap()
        .author(vec![0xff; 20].into())
        .unwrap()
        .group(vec![0xfe; 20].into())
        .unwrap()
        .date(SauceDate::new(2026, 9, 5))
        .file_size(12345)
        .data_type(data_type)
        .file_type(123)
        .t_info1(321)
        .t_info2(456)
        .t_info3(789)
        .t_info4(987)
        .t_flags(0xff)
        .t_info_s(vec![0x82; 22].into())
        .unwrap()
        .add_comment(vec![0xfe; 64].into())
        .unwrap()
        .build()
}

#[test]
fn add_alter_remove_preserves_payload_and_calculates_size() {
    let fixture = Fixture::new(None);
    fixture.run("add", &["--title", "New"]);
    assert_eq!(fixture.record().file_size(), PAYLOAD.len() as u32);
    fixture.run("alter", &["--group", "Group"]);
    assert_eq!(fixture.record().file_size(), PAYLOAD.len() as u32);
    fixture.run("remove", &["--strip-eof"]);
    assert_eq!(fs::read(&fixture.file).unwrap(), PAYLOAD);
}

#[test]
fn alter_preserves_untouched_bytes_and_raw_fields() {
    for data_type in [
        SauceDataType::None,
        SauceDataType::Character,
        SauceDataType::BinaryText,
        SauceDataType::Undefined(99),
    ] {
        let original = unusual_record(data_type);
        let fixture = Fixture::new(Some(&original));
        fixture.run("alter", &["--group", "Changed"]);
        let expected = original
            .to_builder()
            .group("Changed".into())
            .unwrap()
            .build();
        assert!(fixture.record() == expected);

        fixture.run("alter", &["--tinfo1", "42"]);
        let expected = expected.to_builder().t_info1(42).build();
        assert!(fixture.record() == expected);
    }
}

#[test]
fn no_op_alter_preserves_a_canonical_record_byte_for_byte() {
    let fixture = Fixture::new(Some(&unusual_record(SauceDataType::Undefined(99))));
    let original = fs::read(&fixture.file).unwrap();
    fixture.run("alter", &[]);
    assert_eq!(fs::read(&fixture.file).unwrap(), original);
}

#[test]
fn force_add_recalculates_size_without_retaining_old_metadata() {
    let original = unusual_record(SauceDataType::Character);
    let fixture = Fixture::new(Some(&original));
    fixture.run("add", &["--force", "--title", "Replacement"]);
    let record = fixture.record();
    assert_eq!(record.file_size(), PAYLOAD.len() as u32);
    assert!(record.comments().is_empty());
    assert_eq!(record.title(), "Replacement");
}

#[test]
fn json_file_size_is_honored_for_add_and_alter_including_zero() {
    for size in [0, 42, u32::MAX] {
        let fixture = Fixture::new(None);
        let path = fixture.json_path(json!({"file_size": size}));
        fixture.run("add", &["--from-json", path.to_str().unwrap()]);
        assert_eq!(fixture.record().file_size(), size);
        let path = fixture.json_path(json!({"file_size": 17}));
        fixture.run("alter", &["--from-json", path.to_str().unwrap()]);
        assert_eq!(fixture.record().file_size(), 17);
    }
}

#[test]
fn cli_comments_override_json_and_work_with_omitted_json_comments() {
    for metadata in [json!({}), json!({"comments": ["JSON"]})] {
        let fixture = Fixture::new(None);
        let path = fixture.json_path(metadata);
        fixture.run(
            "add",
            &[
                "--from-json",
                path.to_str().unwrap(),
                "--comment",
                "CLI",
                "--comment",
                "Second",
            ],
        );
        let record = fixture.record();
        assert_eq!(record.comments().len(), 2);
        assert_eq!(record.comments()[0], "CLI");
        assert_eq!(record.comments()[1], "Second");
    }
}

#[test]
fn precedence_is_resolved_before_validating_overridden_json_values() {
    let fixture = Fixture::new(None);
    let path = fixture.json_path(json!({
        "title": "x".repeat(100), "date": "invalid", "comments": ["x".repeat(100)],
        "author": "JSON author", "file_size": 42, "data_type": 99, "tinfo1": 321
    }));
    fixture.run(
        "add",
        &[
            "--from-json",
            path.to_str().unwrap(),
            "--title",
            "CLI",
            "--date",
            "2026-09-05",
            "--comment",
            "CLI comment",
            "--tinfo1",
            "80",
        ],
    );
    let record = fixture.record();
    assert_eq!(record.title(), "CLI");
    assert_eq!(record.author(), "JSON author");
    assert_eq!(record.file_size(), 42);
    assert_eq!(record.data_type(), SauceDataType::Undefined(99));
    assert_eq!(record.header().t_info1, 80);
    assert_eq!(record.date(), SauceDate::new(2026, 9, 5));
}

#[test]
fn json_comments_distinguish_omission_empty_and_replacement() {
    let original = unusual_record(SauceDataType::Character);
    let fixture = Fixture::new(Some(&original));
    let path = fixture.json_path(json!({"group": "Changed"}));
    fixture.run("alter", &["--from-json", path.to_str().unwrap()]);
    assert_eq!(fixture.record().comments(), original.comments());

    let path = fixture.json_path(json!({"comments": []}));
    fixture.run("alter", &["--from-json", path.to_str().unwrap()]);
    assert!(fixture.record().comments().is_empty());

    let path = fixture.json_path(json!({"comments": ["JSON"]}));
    fixture.run(
        "alter",
        &[
            "--from-json",
            path.to_str().unwrap(),
            "--add-comment",
            "Appended",
        ],
    );
    assert_eq!(fixture.record().comments().len(), 2);
    assert_eq!(fixture.record().comments()[0], "JSON");
    assert_eq!(fixture.record().comments()[1], "Appended");

    fixture.run(
        "alter",
        &[
            "--from-json",
            path.to_str().unwrap(),
            "--comment",
            "Ignored",
            "--clear-comments",
            "--add-comment",
            "Only this",
        ],
    );
    assert_eq!(fixture.record().comments().len(), 1);
    assert_eq!(fixture.record().comments()[0], "Only this");
}

#[test]
fn invalid_dates_return_errors_without_panics_or_file_changes() {
    for date in [
        "123é456",
        "2026/09/05",
        "2026090x",
        "20260905-",
        "202-609-05",
    ] {
        for operation in ["add", "alter"] {
            let original = unusual_record(SauceDataType::Character);
            let fixture = Fixture::new((operation == "alter").then_some(&original));
            let bytes = fs::read(&fixture.file).unwrap();
            let output = fixture
                .command(operation)
                .args(["--date", date])
                .output()
                .unwrap();
            assert_eq!(output.status.code(), Some(1));
            assert!(!String::from_utf8_lossy(&output.stderr).contains("panicked"));
            assert_eq!(fs::read(&fixture.file).unwrap(), bytes);

            let path = fixture.json_path(json!({"date": date}));
            let output = fixture
                .command(operation)
                .arg("--from-json")
                .arg(path)
                .output()
                .unwrap();
            assert_eq!(output.status.code(), Some(1));
            assert_eq!(fs::read(&fixture.file).unwrap(), bytes);
        }
    }
}

#[test]
fn dates_accept_both_documented_formats_and_unknown_dates() {
    for date in ["20260905", "2026-09-05", "00000000"] {
        let fixture = Fixture::new(None);
        fixture.run("add", &["--date", date]);
        let expected = if date == "00000000" {
            SauceDate::default()
        } else {
            SauceDate::new(2026, 9, 5)
        };
        assert_eq!(fixture.record().date(), expected);
    }
}

#[test]
fn rejected_updates_leave_original_unchanged_and_no_temporary_files() {
    let fixture = Fixture::new(Some(&unusual_record(SauceDataType::Character)));
    let bytes = fs::read(&fixture.file).unwrap();
    let output = fixture
        .command("alter")
        .args(["--title", &"x".repeat(36)])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(fs::read(&fixture.file).unwrap(), bytes);
    assert_eq!(fs::read_dir(fixture.dir.path()).unwrap().count(), 1);
}

#[cfg(unix)]
#[test]
fn atomic_updates_preserve_permissions_and_follow_symlinks() {
    use std::os::unix::fs::{PermissionsExt, symlink};

    let mut fixture = Fixture::new(None);
    let target = fixture.file.clone();
    fs::set_permissions(&target, fs::Permissions::from_mode(0o640)).unwrap();
    let link = fixture.dir.path().join("link.ans");
    symlink(&target, &link).unwrap();
    fixture.file = link;
    fixture.run("add", &["--title", "Via symlink"]);
    fixture.run("alter", &["--group", "Group"]);
    fixture.run("remove", &["--strip-eof"]);
    assert!(
        fs::symlink_metadata(&fixture.file)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert_eq!(
        fs::metadata(&target).unwrap().permissions().mode() & 0o777,
        0o640
    );
    assert_eq!(fs::read(target).unwrap(), PAYLOAD);
    assert_eq!(fs::read_dir(fixture.dir.path()).unwrap().count(), 2);
}
