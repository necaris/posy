mod kvstore;
mod package_db;
mod prelude;
mod resolve;
mod util;
mod vocab;

mod env;
mod output;
mod platform_tags;
mod seek_slice;
#[cfg(test)]
mod test_util;
mod trampolines;
mod tree;

use std::path::Path;

use crate::{env::EnvForest, prelude::*, resolve::Brief};

use clap::Parser;
use resolve::AllowPre;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(flatten)]
    output_args: output::OutputArgs,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    output::init(&cli.output_args);

    let db = package_db::PackageDB::new(
        &vec![
            Url::parse("https://pybi.vorpus.org")?,
            Url::parse("https://pypi.org/simple/")?,
        ],
        PROJECT_DIRS.cache_dir(),
    )?;
    let platform = PybiPlatform::current_platform()?;

    let brief = Brief {
        python: "cpython_unofficial >= 3".try_into().unwrap(),
        requirements: vec![
            "trio".try_into().unwrap(),
            "numpy".try_into().unwrap(),
            "black".try_into().unwrap(),
        ],
        allow_pre: AllowPre::Some(HashSet::new()),
    };
    let blueprint = brief.resolve(&db, &platform, None)?;

    let env_forest = EnvForest::new(Path::new("/tmp/posy-test-forest"))?;
    let env = env_forest.get_env(&db, &blueprint, &platform)?;

    let mut cmd = std::process::Command::new("python");
    cmd.envs(env.env_vars()?);

    if cfg!(unix) {
        use std::os::unix::process::CommandExt;
        Err(cmd.exec())?;
        unreachable!();
    } else {
        // XX FIXME: factor out the windows trampoline code and reuse it here.
        //
        // unwrap() is safe b/c this branch only runs on windows, and Windows doesn't
        // have special exit statuses; that's a special thing for Unix signals.
        std::process::exit(cmd.status()?.code().unwrap());
    }
}
