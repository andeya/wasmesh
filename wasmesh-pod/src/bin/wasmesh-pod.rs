use structopt::{clap::AppSettings, StructOpt};

use wasmesh_pod::serve;
use wasmesh_pod::ServeOpt;

#[derive(StructOpt, Debug)]
#[structopt(global_settings = & [AppSettings::VersionlessSubcommands, AppSettings::ColorAuto, AppSettings::ColoredHelp])]
enum Command {
    /// Serve a command from the package or one of the dependencies
    Serve(ServeOpt),
}

fn main() {
    match Command::from_args() {
        Command::Serve(opt) => {
            serve(opt).unwrap();
        }
    }
}
