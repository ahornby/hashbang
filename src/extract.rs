use anyhow::Error;
use bytes::Bytes;
use tempfile::NamedTempFile;

use std::{
    fs::Permissions,
    io::{Read, Write},
    path::Path,
};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use crate::config::ExtractStep;

// execute chain of extraction steps, resulting in executable files on disk in the cache
pub(crate) fn extract(
    cache_dir: &Path,
    mut data: Bytes,
    steps: Vec<ExtractStep>,
) -> Result<(), Error> {
    let mut tmp_file = NamedTempFile::new_in(cache_dir)?;
    let mut exec_path = None;
    for step in steps {
        match step {
            ExtractStep::MakeExecutable(path) => {
                if cfg!(unix) {
                    std::fs::set_permissions(&tmp_file, Permissions::from_mode(0o755))?;
                }
                exec_path = Some(path);
            }
            ExtractStep::WriteArchive => tmp_file.write_all(&data)?,
            ExtractStep::ZstdDecompress => {
                let mut decoder = zstd::Decoder::new(&data[..])?;
                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed)?;
                data = Bytes::from(decompressed);
                tmp_file.write_all(&data)?
            }
        }
    }

    if let Some(exec_path) = exec_path {
        let path = cache_dir.join(exec_path);
        tmp_file.persist(path)?;
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_extract_zstd_make_exec() {
        let cache_dir = tempfile::tempdir().unwrap();
        let data = Bytes::from(std::fs::read("tests/data/hello_world.sh.zst").unwrap());
        let steps = vec![
            ExtractStep::ZstdDecompress,
            ExtractStep::MakeExecutable("hello_world.sh".to_string()),
        ];
        extract(cache_dir.path(), data, steps).unwrap();
        let path = cache_dir.path().join("hello_world.sh");
        assert!(path.exists());
        if cfg!(unix) {
            let perms = std::fs::metadata(&path).unwrap().permissions();
            assert!(perms.mode() & 0o111 != 0);
        }
    }
}
