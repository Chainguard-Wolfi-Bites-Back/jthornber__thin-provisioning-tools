use anyhow::Result;
use std::str::from_utf8;
use thinp::file_utils;
use thinp::version::tools_version;

mod common;
use common::test_dir::*;
use common::*;

//------------------------------------------

#[test]
fn accepts_v() -> Result<()> {
    let stdout = thin_restore!("-V").read()?;
    assert!(stdout.contains(tools_version()));
    Ok(())
}

#[test]
fn accepts_version() -> Result<()> {
    let stdout = thin_restore!("--version").read()?;
    assert!(stdout.contains(tools_version()));
    Ok(())
}

const USAGE: &str = "Usage: thin_restore [options]\nOptions:\n  {-h|--help}\n  {-i|--input} <input xml file>\n  {-o|--output} <output device or file>\n  {--transaction-id} <natural>\n  {--data-block-size} <natural>\n  {--nr-data-blocks} <natural>\n  {-q|--quiet}\n  {-V|--version}";

#[test]
fn accepts_h() -> Result<()> {
    let stdout = thin_restore!("-h").read()?;
    assert_eq!(stdout, USAGE);
    Ok(())
}

#[test]
fn accepts_help() -> Result<()> {
    let stdout = thin_restore!("--help").read()?;
    assert_eq!(stdout, USAGE);
    Ok(())
}

#[test]
fn missing_input_arg() -> Result<()> {
    let mut td = TestDir::new()?;
    let md = mk_zeroed_md(&mut td)?;
    let stderr = run_fail(thin_restore!("-o", &md))?;
    assert!(stderr.contains(msg::MISSING_INPUT_ARG));
    Ok(())
}

#[test]
fn input_file_not_found() -> Result<()> {
    let mut td = TestDir::new()?;
    let md = mk_zeroed_md(&mut td)?;
    let stderr = run_fail(thin_restore!("-i", "no-such-file", "-o", &md))?;
    assert!(superblock_all_zeroes(&md)?);
    assert!(stderr.contains(msg::FILE_NOT_FOUND));
    Ok(())
}

#[test]
fn garbage_input_file() -> Result<()> {
    let mut td = TestDir::new()?;
    let xml = mk_zeroed_md(&mut td)?;
    let md = mk_zeroed_md(&mut td)?;
    let _stderr = run_fail(thin_restore!("-i", &xml, "-o", &md))?;
    assert!(superblock_all_zeroes(&md)?);
    Ok(())
}

#[test]
fn missing_output_arg() -> Result<()> {
    let mut td = TestDir::new()?;
    let xml = mk_valid_xml(&mut td)?;
    let stderr = run_fail(thin_restore!("-i", &xml))?;
    assert!(stderr.contains(msg::MISSING_OUTPUT_ARG));
    Ok(())
}

#[test]
fn tiny_output_file() -> Result<()> {
    let mut td = TestDir::new()?;
    let xml = mk_valid_xml(&mut td)?;
    let md = td.mk_path("meta.bin");
    let _file = file_utils::create_sized_file(&md, 4096);
    let stderr = run_fail(thin_restore!("-i", &xml, "-o", &md))?;
    assert!(stderr.contains("Output file too small"));
    Ok(())
}

fn quiet_flag(flag: &str) -> Result<()> {
    let mut td = TestDir::new()?;
    let xml = mk_valid_xml(&mut td)?;
    let md = mk_zeroed_md(&mut td)?;

    let output = thin_restore!("-i", &xml, "-o", &md, flag).run()?;

    assert!(output.status.success());
    assert_eq!(output.stdout.len(), 0);
    assert_eq!(output.stderr.len(), 0);
    Ok(())
}

#[test]
fn accepts_q() -> Result<()> {
    quiet_flag("-q")
}

#[test]
fn accepts_quiet() -> Result<()> {
    quiet_flag("--quiet")
}

fn override_something(flag: &str, value: &str, pattern: &str) -> Result<()> {
    let mut td = TestDir::new()?;
    let xml = mk_valid_xml(&mut td)?;
    let md = mk_zeroed_md(&mut td)?;

    thin_restore!("-i", &xml, "-o", &md, flag, value).run()?;

    let output = thin_dump!(&md).run()?;
    assert!(from_utf8(&output.stdout)?.contains(pattern));
    Ok(())
}

#[test]
fn override_transaction_id() -> Result<()> {
    override_something("--transaction-id", "2345", "transaction=\"2345\"")
}

#[test]
fn override_data_block_size() -> Result<()> {
    override_something("--data-block-size", "8192", "data_block_size=\"8192\"")
}

#[test]
fn override_nr_data_blocks() -> Result<()> {
    override_something("--nr-data-blocks", "234500", "nr_data_blocks=\"234500\"")
}
