use clap::{Arg, ArgGroup};
use std::path::Path;
use std::process;

use crate::thin::damage_generator::*;

//------------------------------------------
use crate::commands::Command;

pub struct ThinGenerateDamageCommand;

impl ThinGenerateDamageCommand {
    fn cli<'a>(&self) -> clap::Command<'a> {
        clap::Command::new(self.name())
            .color(clap::ColorChoice::Never)
            .version(crate::version::tools_version())
            .about("A tool for creating synthetic thin metadata.")
            // flags
            .arg(
                Arg::new("ASYNC_IO")
                    .help("Force use of io_uring for synchronous io")
                    .long("async-io")
                    .hide(true),
            )
            .arg(
                Arg::new("CREATE_METADATA_LEAKS")
                    .help("Create leaked metadata blocks")
                    .long("create-metadata-leaks")
                    .requires_all(&["EXPECTED", "ACTUAL", "NR_BLOCKS"])
                    .group("commands"),
            )
            // options
            .arg(
                Arg::new("EXPECTED")
                    .help("The expected reference count of damaged blocks")
                    .long("expected")
                    .value_name("REFCONT"),
            )
            .arg(
                Arg::new("ACTUAL")
                    .help("The actual reference count of damaged blocks")
                    .long("actual")
                    .value_name("REFCOUNT"),
            )
            .arg(
                Arg::new("NR_BLOCKS")
                    .help("Specify the number of metadata blocks")
                    .long("nr-blocks")
                    .value_name("NUM"),
            )
            .arg(
                Arg::new("OUTPUT")
                    .help("Specify the output device")
                    .short('o')
                    .long("output")
                    .value_name("FILE")
                    .required(true),
            )
            .group(ArgGroup::new("commands").required(true))
    }
}

impl<'a> Command<'a> for ThinGenerateDamageCommand {
    fn name(&self) -> &'a str {
        "thin_generate_damage"
    }

    fn run(&self, args: &mut dyn Iterator<Item = std::ffi::OsString>) -> std::io::Result<()> {
        let matches = self.cli().get_matches_from(args);

        let opts = ThinDamageOpts {
            async_io: matches.is_present("ASYNC_IO"),
            op: if matches.is_present("CREATE_METADATA_LEAKS") {
                DamageOp::CreateMetadataLeaks {
                    nr_blocks: matches.value_of_t_or_exit::<usize>("NR_BLOCKS"),
                    expected_rc: matches.value_of_t_or_exit::<u32>("EXPECTED"),
                    actual_rc: matches.value_of_t_or_exit::<u32>("ACTUAL"),
                }
            } else {
                eprintln!("unknown option");
                process::exit(1);
            },
            output: Path::new(matches.value_of("OUTPUT").unwrap()),
        };

        damage_metadata(opts).map_err(|reason| {
            eprintln!("{}", reason);
            std::io::Error::from_raw_os_error(libc::EPERM)
        })
    }
}

//------------------------------------------