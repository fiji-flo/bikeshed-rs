use clap::{App, Arg, ArgMatches, SubCommand};

use crate::spec::Spec;

fn handle_spec(matches: ArgMatches) {
    let infile = matches
        .subcommand_matches("spec")
        .unwrap()
        .value_of("infile")
        .unwrap();
    let outfile = matches
        .subcommand_matches("spec")
        .unwrap()
        .value_of("outfile");

    let mut doc = Spec::new(infile);
    doc.preprocess();
    doc.finish(outfile);
}

pub fn run() {
    let spec_subcommand = SubCommand::with_name("spec")
        .about("Process a spec source file into a valid output file")
        .arg(
            Arg::with_name("infile")
                .required(true)
                .takes_value(true)
                .help("path to the source file")
                .index(1),
        )
        .arg(
            Arg::with_name("outfile")
                .takes_value(true)
                .help("path to the output file")
                .index(2),
        );

    let matches = App::new("Bikeshed.rs Demo")
        .version("1.0")
        .author("whichxjy")
        .subcommand(spec_subcommand)
        .get_matches();

    match matches.subcommand_name() {
        Some("spec") => handle_spec(matches),
        _ => {}
    }
}