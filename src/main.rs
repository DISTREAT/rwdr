use anyhow::Result;
use flate2::read::GzDecoder;
use indicatif::{ProgressBar, ProgressStyle};
use md5_rs::Context;
use mtree::MTree;
use rayon::prelude::*;
use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use walkdir::WalkDir;

fn get_mtree_paths() -> Result<Vec<PathBuf>> {
    let mut mtree_paths: Vec<PathBuf> = vec![];

    for entry in WalkDir::new("/var/lib/pacman/local")
        .into_iter()
        .map(|e| e.unwrap())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.file_name().to_str().unwrap() == "mtree")
    {
        mtree_paths.push(entry.path().to_path_buf());
    }

    Ok(mtree_paths)
}

fn md5sum_of_file(mut file: &File) -> Result<u128> {
    const BUFFER_LEN: usize = 512;

    let mut ctx = Context::new();
    let mut buffer = [0u8; BUFFER_LEN];

    loop {
        let read_count = file.read(&mut buffer)?;
        ctx.read(&buffer[..read_count]);

        if read_count != BUFFER_LEN {
            break;
        }
    }

    let digest = u128::from_be_bytes(ctx.finish());

    Ok(digest)
}

fn get_recreatable_paths_from_mtree(path: &PathBuf) -> Result<Vec<String>> {
    let mut recreatable_paths = Vec::<String>::new();

    let file = File::open(path)?;
    let decompression_stream = GzDecoder::new(file);

    for entry in MTree::from_reader(decompression_stream)
        .map(|e| e.unwrap())
        .filter(|e| e.file_type().unwrap() == mtree::FileType::File)
    {
        let entry_path = PathBuf::from("/").join(entry.path().strip_prefix("./")?);
        let file = match File::open(&entry_path) {
            Ok(file) => file,
            Err(_error) => continue, // ignore as non-recreatable (includes internal build files; not optimal)
        };
        let digest = md5sum_of_file(&file)?;

        if entry.md5().unwrap() == digest {
            recreatable_paths.push(entry_path.display().to_string());
        }
    }

    Ok(recreatable_paths)
}

fn get_recreatable_paths() -> Result<HashSet<String>> {
    let mtree_paths = get_mtree_paths()?;
    let progressbar = ProgressBar::new(mtree_paths.len().try_into().unwrap());
    progressbar.set_style(
        ProgressStyle::with_template("[{elapsed_precise} - {pos}/{len}] {msg}").unwrap(),
    );

    let recreatable_paths: HashSet<String> = mtree_paths
        .par_iter()
        .map(|mtree_path| {
            (
                mtree_path,
                get_recreatable_paths_from_mtree(&mtree_path).unwrap(),
            )
        })
        .inspect(|e| {
            progressbar.inc(1);
            progressbar.set_message(format!("{}", e.0.iter().nth(5).unwrap().to_str().unwrap()));
        })
        .map(|e| e.1)
        .flatten()
        .collect();

    progressbar.finish();

    Ok(recreatable_paths)
}

fn main() -> Result<()> {
    let recreatable_paths = get_recreatable_paths()?;

    for root_path in vec!["/etc", "/usr", "/boot", "/opt", "/srv", "var"] {
        for entry in WalkDir::new(&root_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| !e.path().starts_with("/var/lib/pacman/local"))
        {
            let path = entry.path().display().to_string();
            if !recreatable_paths.contains(&path) {
                println!("{}", path);
            }
        }
    }

    Ok(())
}
