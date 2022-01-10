extern crate clap;

use clap::{App, Arg};
use std::path::Path;
use std::process;

use crate::commands::utils::*;
use crate::thin::metadata_repair::SuperblockOverrides;
use crate::thin::restore::{restore, ThinRestoreOptions};

pub fn run(args: &[std::ffi::OsString]) {
    let parser = App::new("thin_restore")
        .version(crate::version::tools_version())
        .about("Convert XML format metadata to binary.")
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
        // options
        .arg(
            Arg::with_name("DATA_BLOCK_SIZE")
                .help("Override the data block size if needed")
                .long("data-block-size")
                .value_name("SECTORS"),
        )
        .arg(
            Arg::with_name("INPUT")
                .help("Specify the input xml")
                .short("i")
                .long("input")
                .value_name("FILE")
                .required(true),
        )
        .arg(
            Arg::with_name("NR_DATA_BLOCKS")
                .help("Override the number of data blocks if needed")
                .long("nr-data-blocks")
                .value_name("NUM"),
        )
        .arg(
            Arg::with_name("OUTPUT")
                .help("Specify the output device")
                .short("o")
                .long("output")
                .value_name("FILE")
                .required(true),
        )
        .arg(
            Arg::with_name("TRANSACTION_ID")
                .help("Override the transaction id if needed")
                .long("transaction-id")
                .value_name("NUM"),
        );

    let matches = parser.get_matches_from(args);
    let input_file = Path::new(matches.value_of("INPUT").unwrap());
    let output_file = Path::new(matches.value_of("OUTPUT").unwrap());

    let report = mk_report(matches.is_present("QUIET"));
    check_input_file(input_file, &report);
    check_output_file(output_file, &report);

    let transaction_id = matches.value_of("TRANSACTION_ID").map(|s| {
        s.parse::<u64>().unwrap_or_else(|_| {
            report.fatal("Couldn't parse transaction_id");
            process::exit(1);
        })
    });

    let data_block_size = matches.value_of("DATA_BLOCK_SIZE").map(|s| {
        s.parse::<u32>().unwrap_or_else(|_| {
            report.fatal("Couldn't parse data_block_size");
            process::exit(1);
        })
    });

    let nr_data_blocks = matches.value_of("NR_DATA_BLOCKS").map(|s| {
        s.parse::<u64>().unwrap_or_else(|_| {
            report.fatal("Couldn't parse nr_data_blocks");
            process::exit(1);
        })
    });

    let opts = ThinRestoreOptions {
        input: input_file,
        output: output_file,
        async_io: matches.is_present("ASYNC_IO"),
        report: report.clone(),
        overrides: SuperblockOverrides {
            transaction_id,
            data_block_size,
            nr_data_blocks,
        },
    };

    if let Err(reason) = restore(opts) {
        report.fatal(&format!("{}", reason));
        process::exit(1);
    }
}
