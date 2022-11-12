pub mod opts;

use common_failures::Result;
use crev_recursive_digest;
use digest::Digest;
use failure::bail;
use failure::{format_err, ResultExt};
use glob;
use log::{debug, info, trace};
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;

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
            res.push(std::mem::take(&mut prev_line))
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
            let mut src_globs = &src_globs[..src_globs.len() - 1];
            trace!("Src globs: {:?}", src_globs);

            if let Some(first_src) = src_globs.get(0) {
                // Skip `--chown=...`, just to not have to log it doesn't
                // match anything
                if first_src.starts_with("--chown=") {
                    src_globs = &src_globs[1..]
                }
            }

            if let Some(first_src) = src_globs.get(0) {
                // Skip `COPY --from=...` entirely, since it doesn't refer
                // to external files
                if first_src.starts_with("--from=") {
                    return Ok(res);
                }
            }

            for src_glob in src_globs {
                let matches: Vec<_> = glob::glob(src_glob)?.collect();
                if matches.is_empty() {
                    info!("{} glob did not match any files", src_glob);
                }
                for entry in matches {
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

fn get_paths_from_dockerfile(content: &str) -> Result<Vec<PathBuf>> {
    let mut res = vec![];

    for line in get_coalesced_lines_from_dockerfile_content(content.into())? {
        res.append(&mut get_external_paths_from_dockerfile_line(line)?);
    }

    Ok(res)
}

#[cfg(unix)]
fn metadata_to_u16(metadata: &std::fs::Metadata) -> u16 {
    let permissions = metadata.permissions();
    use std::os::unix::fs::PermissionsExt;
    (permissions.mode() & 0x1ff) as u16
}

#[cfg(not(unix))]
fn metadata_to_u16(metadata: &std::fs::Metadata) -> u16 {
    let permissions = metadata.permissions();
    // TODO: what else to do on Windows?
    match (permissions.readonly(), metadata.is_dir()) {
        (false, false) => 0o444u16,
        (false, true) => 0o555,
        (true, false) => 0x666,
        (true, true) => 0x777,
    }
}

pub fn hash(dockerfile_path: &Path, opts: opts::Opts) -> Result<String> {
    debug!("Opening dockerfile: {}", dockerfile_path.display());
    let dockerfile_content = std::fs::read_to_string(&dockerfile_path).with_context(|_| {
        format_err!("Could not read dockerfile: {}", dockerfile_path.display())
    })?;

    std::env::set_current_dir(&opts.context_path).with_context(|_e| {
        format_err!(
            "Couldn't cd to context dir: {}",
            opts.context_path.display()
        )
    })?;

    let rel_ignore_paths: HashSet<_> = opts.ignore_path.clone().into_iter().collect();

    let paths_from_dockerfile = get_paths_from_dockerfile(&dockerfile_content)?;

    for path in &paths_from_dockerfile {
        info!("Dockerfile depends on: {}", path.display());
    }

    let mut digests = paths_from_dockerfile
        .into_iter()
        .chain(opts.extra_path)
        .map(|path| {
            let rdigest = crev_recursive_digest::RecursiveDigest::<blake2::Blake2b, _, _>::new()
                .additional_data(|entry, writer| {
                    let metadata = entry.metadata()?;
                    writer.input(&metadata_to_u16(&metadata).to_be_bytes());
                    Ok(())
                })
                .filter(|entry| {
                    let rel_path = entry.path().strip_prefix(&path).expect("must be prefix");
                    !rel_ignore_paths.contains(rel_path)
                })
                .build();
            let digest = rdigest.get_digest_of(&path);

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

    digests.push(dockerfile_content.as_bytes().to_vec());

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
    Ok(format!(
        "{}",
        if opts.hex {
            hex::encode(&digest)
        } else {
            base64::encode_config(&digest, base64::URL_SAFE_NO_PAD)
        }
    ))
}
