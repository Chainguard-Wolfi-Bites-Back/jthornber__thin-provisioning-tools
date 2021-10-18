use anyhow::Result;

mod common;

use common::common_args::*;
use common::fixture::*;
use common::input_arg::*;
use common::process::*;
use common::program::*;
use common::target::*;
use common::test_dir::*;
use common::thin::*;

//------------------------------------------

const USAGE: &str = "thin_check 0.9.0
Validates thin provisioning metadata on a device or file.

USAGE:
    thin_check [FLAGS] [OPTIONS] <INPUT>

FLAGS:
        --auto-repair                Auto repair trivial issues.
        --clear-needs-check-flag     Clears the 'needs_check' flag in the superblock
        --ignore-non-fatal-errors    Only return a non-zero exit code if a fatal error is found.
    -m, --metadata-snapshot          Check the metadata snapshot on a live pool
    -q, --quiet                      Suppress output messages, return only exit code.
        --super-block-only           Only check the superblock.
        --skip-mappings              Don't check the mapping tree
    -h, --help                       Prints help information
    -V, --version                    Prints version information

OPTIONS:
        --override-mapping-root <OVERRIDE_MAPPING_ROOT>    Specify a mapping root to use

ARGS:
    <INPUT>    Specify the input device to check";

//-----------------------------------------

struct ThinCheck;

impl<'a> Program<'a> for ThinCheck {
    fn name() -> &'a str {
        "thin_check"
    }

    fn cmd<I>(args: I) -> Command
    where
        I: IntoIterator,
        I::Item: Into<std::ffi::OsString>,
    {
        thin_check_cmd(args)
    }

    fn usage() -> &'a str {
        USAGE
    }

    fn arg_type() -> ArgType {
        ArgType::InputArg
    }

    fn bad_option_hint(option: &str) -> String {
        msg::bad_option_hint(option)
    }
}

impl<'a> InputProgram<'a> for ThinCheck {
    fn mk_valid_input(td: &mut TestDir) -> Result<std::path::PathBuf> {
        mk_valid_md(td)
    }

    fn file_not_found() -> &'a str {
        msg::FILE_NOT_FOUND
    }

    fn missing_input_arg() -> &'a str {
        msg::MISSING_INPUT_ARG
    }

    fn corrupted_input() -> &'a str {
        msg::BAD_SUPERBLOCK
    }
}

impl<'a> MetadataReader<'_> for ThinCheck {}

//------------------------------------------

test_accepts_help!(ThinCheck);
test_accepts_version!(ThinCheck);
test_rejects_bad_option!(ThinCheck);

test_missing_input_arg!(ThinCheck);
test_input_file_not_found!(ThinCheck);
test_input_cannot_be_a_directory!(ThinCheck);
test_unreadable_input_file!(ThinCheck);

test_help_message_for_tiny_input_file!(ThinCheck);
test_spot_xml_data!(ThinCheck);
test_corrupted_input_data!(ThinCheck);

//------------------------------------------
// test exclusive flags

fn accepts_flag(flag: &str) -> Result<()> {
    let mut td = TestDir::new()?;
    let md = mk_valid_md(&mut td)?;
    run_ok(thin_check_cmd(args![flag, &md]))?;
    Ok(())
}

#[test]
fn accepts_superblock_only() -> Result<()> {
    accepts_flag("--super-block-only")
}

#[test]
fn accepts_skip_mappings() -> Result<()> {
    accepts_flag("--skip-mappings")
}

#[test]
fn accepts_ignore_non_fatal_errors() -> Result<()> {
    accepts_flag("--ignore-non-fatal-errors")
}

#[test]
fn accepts_clear_needs_check_flag() -> Result<()> {
    accepts_flag("--clear-needs-check-flag")
}

#[test]
fn accepts_auto_repair() -> Result<()> {
    accepts_flag("--auto-repair")
}

//------------------------------------------
// test the --quiet flag

#[test]
fn accepts_quiet() -> Result<()> {
    let mut td = TestDir::new()?;
    let md = mk_valid_md(&mut td)?;

    let output = run_ok_raw(thin_check_cmd(args!["--quiet", &md]))?;
    if output.stdout.len() > 0 {
        eprintln!("stdout: {:?}", &std::str::from_utf8(&output.stdout));
    }

    if output.stderr.len() > 0 {
        eprintln!("stderr: {:?}", &std::str::from_utf8(&output.stderr));
    }
    assert_eq!(output.stdout.len(), 0);
    assert_eq!(output.stderr.len(), 0);
    Ok(())
}

//------------------------------------------
// test superblock-block-only

#[test]
fn detects_corrupt_superblock_with_superblock_only() -> Result<()> {
    let mut td = TestDir::new()?;
    let md = mk_zeroed_md(&mut td)?;
    let _stderr = run_fail(thin_check_cmd(args!["--super-block-only", &md]))?;
    Ok(())
}

//------------------------------------------
// test info outputs

#[test]
fn prints_info_fields() -> Result<()> {
    let mut td = TestDir::new()?;
    let md = mk_valid_md(&mut td)?;
    let stdout = run_ok(thin_check_cmd(args![&md]))?;
    eprintln!("info: {:?}", stdout);
    assert!(stdout.contains("TRANSACTION_ID="));
    assert!(stdout.contains("METADATA_FREE_BLOCKS="));
    Ok(())
}

//------------------------------------------
// test compatibility between options

#[test]
fn auto_repair_incompatible_opts() -> Result<()> {
    let mut td = TestDir::new()?;
    let md = mk_valid_md(&mut td)?;
    run_fail(thin_check_cmd(args!["--auto-repair", "-m", &md]))?;
    run_fail(thin_check_cmd(args![
        "--auto-repair",
        "--override-mapping-root",
        "123",
        &md
    ]))?;
    run_fail(thin_check_cmd(args![
        "--auto-repair",
        "--super-block-only",
        &md
    ]))?;
    run_fail(thin_check_cmd(args![
        "--auto-repair",
        "--skip-mappings",
        &md
    ]))?;
    run_fail(thin_check_cmd(args![
        "--auto-repair",
        "--ignore-non-fatal-errors",
        &md
    ]))?;
    Ok(())
}

#[test]
fn clear_needs_check_incompatible_opts() -> Result<()> {
    let mut td = TestDir::new()?;
    let md = mk_valid_md(&mut td)?;
    run_fail(thin_check_cmd(args!["--clear-needs-check-flag", "-m", &md]))?;
    run_fail(thin_check_cmd(args![
        "--clear-needs-check-flag",
        "--override-mapping-root",
        "123",
        &md
    ]))?;
    run_fail(thin_check_cmd(args![
        "--clear-needs-check-flag",
        "--super-block-only",
        &md
    ]))?;
    run_fail(thin_check_cmd(args![
        "--clear-needs-check-flag",
        "--ignore-non-fatal-errors",
        &md
    ]))?;
    Ok(())
}

//------------------------------------------
// test clear-needs-check

#[test]
fn clear_needs_check() -> Result<()> {
    let mut td = TestDir::new()?;
    let md = prep_metadata(&mut td)?;

    set_needs_check(&md)?;

    assert!(get_needs_check(&md)?);
    run_ok(thin_check_cmd(args!["--clear-needs-check-flag", &md]))?;
    assert!(!get_needs_check(&md)?);
    Ok(())
}

#[test]
fn no_clear_needs_check_if_error() -> Result<()> {
    let mut td = TestDir::new()?;
    let md = prep_metadata(&mut td)?;

    set_needs_check(&md)?;
    generate_metadata_leaks(&md, 1, 0, 1)?;
    run_fail(thin_check_cmd(args!["--clear-needs-check-flag", &md]))?;
    assert!(get_needs_check(&md)?);
    Ok(())
}

#[test]
fn clear_needs_check_if_skip_mappings() -> Result<()> {
    let mut td = TestDir::new()?;
    let md = prep_metadata(&mut td)?;

    set_needs_check(&md)?;
    generate_metadata_leaks(&md, 1, 0, 1)?;

    assert!(get_needs_check(&md)?);
    run_ok(thin_check_cmd(args![
        "--clear-needs-check-flag",
        "--skip-mappings",
        &md
    ]))?;
    assert!(!get_needs_check(&md)?);
    Ok(())
}

//------------------------------------------
// test ignore-non-fatal-errors

#[test]
fn metadata_leaks_are_non_fatal() -> Result<()> {
    let mut td = TestDir::new()?;
    let md = prep_metadata(&mut td)?;
    generate_metadata_leaks(&md, 1, 0, 1)?;
    run_fail(thin_check_cmd(args![&md]))?;
    run_ok(thin_check_cmd(args!["--ignore-non-fatal-errors", &md]))?;
    Ok(())
}

#[test]
fn fatal_errors_cant_be_ignored() -> Result<()> {
    let mut td = TestDir::new()?;
    let md = prep_metadata(&mut td)?;

    generate_metadata_leaks(&md, 1, 1, 0)?;
    ensure_untouched(&md, || {
        run_fail(thin_check_cmd(args!["--ignore-non-fatal-errors", &md]))?;
        Ok(())
    })
}

//------------------------------------------
// test auto-repair

#[test]
fn auto_repair() -> Result<()> {
    let mut td = TestDir::new()?;
    td.dont_clean_up();
    let md = prep_metadata(&mut td)?;

    eprintln!("here 0");

    // auto-repair should have no effect on good metadata.
    ensure_untouched(&md, || {
        // run_ok(thin_check_cmd(args!["--auto-repair", &md]))?;
        run_ok(thin_check_cmd(args![&md]))?;
        Ok(())
    })?;
    eprintln!("here 0.5");

    generate_metadata_leaks(&md, 16, 0, 1)?;
    eprintln!("here 1");
    run_fail(thin_check_cmd(args![&md]))?;
    eprintln!("here 2");
    run_ok(thin_check_cmd(args!["--auto-repair", &md]))?;
    eprintln!("here 3");

    run_ok(thin_check_cmd(args![&md]))?;
    Ok(())
}

#[test]
fn auto_repair_has_limits() -> Result<()> {
    let mut td = TestDir::new()?;
    let md = prep_metadata(&mut td)?;

    generate_metadata_leaks(&md, 16, 1, 0)?;
    ensure_untouched(&md, || {
        run_fail(thin_check_cmd(args!["--auto-repair", &md]))?;
        Ok(())
    })?;
    Ok(())
}

#[test]
fn auto_repair_clears_needs_check() -> Result<()> {
    let mut td = TestDir::new()?;
    let md = prep_metadata(&mut td)?;

    set_needs_check(&md)?;
    run_ok(thin_check_cmd(args!["--auto-repair", &md]))?;
    assert!(!get_needs_check(&md)?);
    Ok(())
}

//------------------------------------------
