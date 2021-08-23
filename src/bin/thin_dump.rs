extern crate clap;
extern crate thinp;

use atty::Stream;
use clap::{App, Arg};
use std::path::Path;
use std::process;
use std::process::exit;
use std::sync::Arc;
use thinp::file_utils;
use thinp::report::*;
use thinp::thin::dump::{dump, ThinDumpOptions};
use thinp::thin::metadata_repair::SuperblockOverrides;

fn main() {
    let parser = App::new("thin_dump")
        .version(thinp::version::tools_version())
        .about("Dump thin-provisioning metadata to stdout in XML format")
        // flags
        .arg(
            Arg::with_name("ASYNC_IO")
                .help("Force use of io_uring for synchronous io")
                .long("async-io")
                .hidden(true),
        )
        .arg(
            Arg::with_name("QUIET")
                .help("Suppress output messages, return only exit code.")
                .short("q")
                .long("quiet"),
        )
        .arg(
            Arg::with_name("REPAIR")
                .help("Repair the metadata whilst dumping it")
                .short("r")
                .long("repair"),
        )
        .arg(
            Arg::with_name("SKIP_MAPPINGS")
                .help("Do not dump the mappings")
                .long("skip-mappings"),
        )
        // options
        .arg(
            Arg::with_name("DATA_BLOCK_SIZE")
                .help("Provide the data block size for repairing")
                .long("data-block-size")
                .value_name("SECTORS"),
        )
        .arg(
            Arg::with_name("METADATA_SNAPSHOT")
                .help("Access the metadata snapshot on a live pool")
                .short("m")
                .long("metadata-snapshot")
                .value_name("METADATA_SNAPSHOT"),
        )
        .arg(
            Arg::with_name("NR_DATA_BLOCKS")
                .help("Override the number of data blocks if needed")
                .long("nr-data-blocks")
                .value_name("NUM"),
        )
        .arg(
            Arg::with_name("OUTPUT")
                .help("Specify the output file rather than stdout")
                .short("o")
                .long("output")
                .value_name("FILE"),
        )
        .arg(
            Arg::with_name("TRANSACTION_ID")
                .help("Override the transaction id if needed")
                .long("transaction-id")
                .value_name("NUM"),
        )
        // arguments
        .arg(
            Arg::with_name("INPUT")
                .help("Specify the input device to dump")
                .required(true)
                .index(1),
        );

    let matches = parser.get_matches();
    let input_file = Path::new(matches.value_of("INPUT").unwrap());
    let output_file = if matches.is_present("OUTPUT") {
        Some(Path::new(matches.value_of("OUTPUT").unwrap()))
    } else {
        None
    };

    if !file_utils::file_exists(input_file) {
        eprintln!("Couldn't find input file '{:?}'.", &input_file);
        exit(1);
    }

    let transaction_id = matches.value_of("TRANSACTION_ID").map(|s| {
        s.parse::<u64>().unwrap_or_else(|_| {
            eprintln!("Couldn't parse transaction_id");
            exit(1);
        })
    });

    let data_block_size = matches.value_of("DATA_BLOCK_SIZE").map(|s| {
        s.parse::<u32>().unwrap_or_else(|_| {
            eprintln!("Couldn't parse data_block_size");
            exit(1);
        })
    });

    let nr_data_blocks = matches.value_of("NR_DATA_BLOCKS").map(|s| {
        s.parse::<u64>().unwrap_or_else(|_| {
            eprintln!("Couldn't parse nr_data_blocks");
            exit(1);
        })
    });

    let report;

    if matches.is_present("QUIET") {
        report = std::sync::Arc::new(mk_quiet_report());
    } else if atty::is(Stream::Stdout) {
        report = std::sync::Arc::new(mk_progress_bar_report());
    } else {
        report = Arc::new(mk_simple_report());
    }

    let opts = ThinDumpOptions {
        input: input_file,
        output: output_file,
        async_io: matches.is_present("ASYNC_IO"),
        report,
        repair: matches.is_present("REPAIR"),
        overrides: SuperblockOverrides {
            transaction_id,
            data_block_size,
            nr_data_blocks,
        },
    };

    if let Err(reason) = dump(opts) {
        println!("{}", reason);
        process::exit(1);
    }
}
