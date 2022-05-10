use bzip2::read::BzDecoder;
use error_chain::error_chain;
use flate2::read::GzDecoder;
use std::fs::{create_dir, File};
use std::io::{copy, Cursor, Read};
use tar::Archive;
use xz2::read::XzDecoder;

error_chain! {
     foreign_links {
         Io(std::io::Error);
         HttpRequest(reqwest::Error);
     }
}

pub async fn download_source_tarball(
    from_url: &String,
    package_name: &String,
) -> Result<Vec<String>> {
    let path = format!("spear_build_{}", package_name);
    create_dir(&path).unwrap();
    let target = from_url;
    let response = reqwest::get(target).await?;
    let fname = response
        .url()
        .path_segments()
        .and_then(std::iter::Iterator::last)
        .and_then(|name| if name.is_empty() { None } else { Some(name) })
        .unwrap_or("source.tar");
    let fname = format!("{}/{}", path, fname);
    let mut dest = File::create(&fname)?;
    let mut content = Cursor::new(response.bytes().await?);
    copy(&mut content, &mut dest)?;
    Ok(vec![path, fname])
}

pub fn extract_source_tarball(
    using: &str,
    to_decompress: &String,
    to_where: &String,
) -> Result<()> {
    let compressed = File::open(&to_decompress)?;
    let to_unpack = match using {
        "gz" => Box::new(GzDecoder::new(compressed)) as Box<dyn Read>,
        "xz" => Box::new(XzDecoder::new(compressed)) as Box<dyn Read>,
        "bz2" => Box::new(BzDecoder::new(compressed)) as Box<dyn Read>,
        "none" => Box::new(compressed) as Box<dyn Read>,
        &_ => {
            panic!("Compression method unsupported; is the package file misconfigured?")
        }
    };
    let mut archive = Archive::new(to_unpack);
    archive.unpack(to_where)?;
    Ok(())
}
