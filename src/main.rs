mod opts;

use common_failures::display::DisplayCausesAndBacktraceExt;
use common_failures::Result;
use crev_recursive_digest;
use digest::Digest;
use failure::bail;
use failure::{format_err, ResultExt};
use glob;
use log::{debug, info, trace};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

fn get_coalesced_lines_from_dockerfile_content(content: String) -> Result<Vec<String>> {
    let mut res = vec![];
    let mut prev_line = String::from("");

    for line in content
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.starts_with("#"))
    {
        if line.ends_with("\\") {
            prev_line.push_str(&line[..line.len() - 1]);
        } else {
            prev_line.push_str(line);
            res.push(std::mem::replace(&mut prev_line, "".into()))
        }
    }

    if prev_line != "" {
        bail!("Trailing \\");
    }

    Ok(res
        .into_iter()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect())
}

/// TODO: Bug, does not support `[src1, src2,... dst]` syntax
fn get_external_paths_from_dockerfile_line(line: String) -> Result<Vec<PathBuf>> {
    trace!("Long Line: {}", line);
    let mut res = vec![];

    let mut parts = line.split_ascii_whitespace();
    let cmd = parts.next().expect("At least the command");

    match cmd {
        "COPY" | "ADD" => {
            let src_globs: Vec<_> = parts.map(str::to_string).collect();
            if src_globs.is_empty() {
                bail!("No arguments to {} command?", cmd);
            }
            let src_globs = &src_globs[..src_globs.len() - 1];
            for src_glob in src_globs {
                for entry in glob::glob(src_glob)? {
                    let entry = entry?;
                    debug!("Matching path found: {}", entry.display());
                    res.push(entry);
                }
            }
        }
        _ => {}
    }

    Ok(res)
}

fn get_paths_from_dockerfile(path: &Path) -> Result<Vec<PathBuf>> {
    debug!("Opening dockerfile: {}", path.display());
    let content = std::fs::read_to_string(path)
        .with_context(|_| format_err!("Could not read dockerfile: {}", path.display()))?;

    let mut res = vec![];

    for line in get_coalesced_lines_from_dockerfile_content(content)? {
        res.append(&mut get_external_paths_from_dockerfile_line(line)?);
    }

    Ok(res)
}

fn run() -> Result<()> {
    env_logger::init();
    let opts = opts::Opts::from_args();

    std::env::set_current_dir(&opts.context_path).with_context(|_e| {
        format_err!(
            "Couldn't cd to context dir: {}",
            opts.context_path.display()
        )
    })?;

    let rel_ignore_paths: HashSet<_> = opts.ignore_path.clone().into_iter().collect();

    let dockerfile_path = opts
        .dockerfile_path
        .clone()
        .unwrap_or_else(|| PathBuf::from("Dockerfile"));

    let paths_from_dockerfile = get_paths_from_dockerfile(&dockerfile_path)?;

    for path in &paths_from_dockerfile {
        info!("Dockerfile depends on: {}", path.display());
    }

    let mut digests = paths_from_dockerfile
        .into_iter()
        .chain(opts.extra_path)
        .chain(vec![dockerfile_path])
        .map(|path| {
            let digest = crev_recursive_digest::get_recursive_digest_for_dir::<blake2::Blake2b, _>(
                &path,
                &rel_ignore_paths,
            );

            if let Ok(ref digest) = digest {
                debug!(
                    "Partial digest: {} for {}",
                    base64::encode_config(&digest, base64::URL_SAFE_NO_PAD),
                    path.display()
                );
            }

            digest
        })
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| format_err!("{}", e))?;

    digests.append(
        &mut opts
            .extra_string
            .iter()
            .map(|s| s.as_bytes().to_vec())
            .collect(),
    );

    digests.sort();

    let mut hasher = blake2::Blake2b::new();
    for digest in &digests {
        debug!(
            "Sorted chunk: {}",
            base64::encode_config(&digest, base64::URL_SAFE_NO_PAD)
        );
        hasher.input(&digest);
    }

    let digest = &hasher.result().to_vec();
    println!(
        "{}",
        if opts.hex {
            hex::encode(&digest)
        } else {
            base64::encode_config(&digest, base64::URL_SAFE_NO_PAD)
        }
    );

    Ok(())
}

fn main() {
    match run() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e.display_causes_and_backtrace());
            std::process::exit(-2)
        }
    }
}
