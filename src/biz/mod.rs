use std::collections::HashSet;
use std::path::PathBuf;
use std::process::exit;
use std::str;
use std::{fs, path::Path};

use anyhow::Context;
use colored::Colorize;
use flate2::read::GzDecoder;
use futures_util::StreamExt;

use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Url;

use crate::preset;
use crate::preset::{ffg_home, ffg_mirror, pkgs};
use async_zip::base::read::seek::ZipFileReader;
use sha256::try_async_digest;
use tar::Archive;
use tokio::fs::{create_dir_all, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::{fs::File, io::BufReader};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

pub struct CommandAction {
    // todo
}

impl CommandAction {
    /// just install specify version
    pub async fn ins(version: &str) -> anyhow::Result<()> {
        println!("install {}", version.green().bold());
        let local_versions = CommandAction::local_version().await?;
        if local_versions.contains(version) {
            println!("already installed {}", version.green());
            return Ok(());
        }
        let remote_version_meta = CommandAction::ls_remote_internal().await?;
        if remote_version_meta
            .iter()
            .find(|item| item.version == version)
            .take()
            .is_none()
        {
            anyhow::bail!(format!(
                "not found go version {}, please use command {} before install specify version",
                version.red().bold(),
                "ffg ls-remote".red().bold()
            ))
        }

        let file_name = CommandUtil::file_name(version);

        let data = remote_version_meta
            .iter()
            .find(|item| item.version == version)
            .unwrap()
            .package
            .iter()
            .find(|item| item.file_name == file_name)
            .unwrap();

        let mirror = ffg_mirror.clone();

        let url = Url::parse(&mirror)?.join(&data.path)?;
        println!("downloading pkg {}", url.to_string().green());

        let save_path = Path::new(&ffg_home.clone())
            .join(pkgs.clone())
            .join(&file_name);

        CommandUtil::download(url.as_str(), save_path.to_string_lossy().as_ref()).await?;
        let sha256 = CommandUtil::sum_sha256(save_path.to_string_lossy().as_ref()).await?;
        if !sha256.eq(&data.sha256_checksum) {
            anyhow::bail!(format!("checksum not pass"))
        }
        CommandUtil::unpack_file(save_path.to_string_lossy().as_ref()).await?;
        let src_dir = Path::new(&ffg_home.clone())
            .join(preset::pkgs.clone())
            .join("go");
        let dst_dir = Path::new(&ffg_home.clone())
            .join(preset::pkgs.clone())
            .join(format!("go{}", version));
        if dst_dir.exists() {
            std::fs::remove_dir_all(&dst_dir)?;
        }
        std::fs::rename(src_dir, &dst_dir)?;
        Ok(())
    }
}

impl CommandAction {
    /// remove specify version
    pub async fn rm(version: &str) -> anyhow::Result<()> {
        let curr_version = CommandAction::current_version().await?;
        let file_name = CommandUtil::file_name(version);
        let zip_file = Path::new(&ffg_home.clone())
            .join(pkgs.clone())
            .join(file_name);
        if zip_file.exists() {
            std::fs::remove_file(zip_file)?;
        }
        let del_version_path = Path::new(&ffg_home.clone())
            .join(pkgs.clone())
            .join(format!("go{}", version));
        if !del_version_path.exists() {
            println!("not found version {}", version.red().bold());
            exit(1);
        }

        // check symlink
        if curr_version.eq(version) {
            let sym_link = Path::new(&ffg_home.clone()).join("go");
            symlink::remove_symlink_dir(&sym_link)?;
            std::fs::remove_dir_all(del_version_path)?;
        } else {
            std::fs::remove_dir_all(del_version_path)?;
        }
        println!("remove {}", version.red());
        Ok(())
    }
}

impl CommandAction {
    /// current used version
    async fn current_version() -> anyhow::Result<String> {
        let current_version_path = Path::new(&ffg_home.clone()).join("go");

        let mut current_version: String = "".to_owned();
        if current_version_path.exists() && current_version_path.is_symlink() {
            let res = current_version_path.read_link();
            current_version = res
                .unwrap()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .replace("go", "");
        }
        Ok(current_version)
    }
}

impl CommandAction {
    /// fetch released version
    pub async fn ls_remote() -> anyhow::Result<()> {
        let curr_version = CommandAction::current_version().await?;
        let local_versions = CommandAction::local_version().await?;
        let remote_versions = CommandAction::ls_remote_internal().await?;
        remote_versions.iter().for_each(|item| {
            if curr_version == item.version {
                println!("*{} current", item.version.bold().green());
            } else if local_versions.contains(&item.version) {
                println!("{}", item.version.magenta());
            } else {
                println!("{}", item.version.bold());
            }
        });

        Ok(())
    }
}

impl CommandAction {
    /// list installed version
    pub async fn ls() -> anyhow::Result<()> {
        let local_version = CommandAction::local_version().await?;
        let current_version_path = Path::new(&ffg_home.clone()).join("go");
        let mut current_version: String = "".to_owned();
        if current_version_path.exists() && current_version_path.is_symlink() {
            let res = current_version_path.read_link();
            current_version = res
                .unwrap()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .replace("go", "");
        }

        local_version.iter().for_each(|f| {
            if current_version.contains(f) {
                println!("*{} {}", f.green(), "current".to_owned())
            } else {
                println!("{}", f);
            }
        });
        Ok(())
    }
}

impl CommandAction {
    /// use specific version, if not installed will install and then check out
    pub async fn use_action(version: &str) -> anyhow::Result<()> {
        println!("use {}", version.bold().green());

        let dst_dir = Path::new(&ffg_home.clone())
            .join(preset::pkgs.clone())
            .join(format!("go{}", version));

        CommandAction::ins(version).await?;
        let soft_link = Path::new(&ffg_home.clone()).join("go");
        if soft_link.exists() {
            symlink::remove_symlink_dir(&soft_link)?;
        }

        #[cfg(target_os = "windows")]
        {
            symlink::symlink_dir(&dst_dir, &soft_link)?;
        }

        #[cfg(not(target_os = "windows"))]
        {
            symlink::symlink_dir(&dst_dir, &soft_link)?;
        }

        Ok(())
    }
}

impl CommandAction {
    async fn ls_remote_internal() -> anyhow::Result<Vec<VersionMeta>> {
        let mirror = preset::ffg_mirror.clone();
        let dl_page_url = Url::parse(&mirror)?.join("dl")?;
        println!(
            "fetch go version's metadata from {}",
            dl_page_url.as_str().bold().green()
        );
        let body = reqwest::get(dl_page_url.as_str()).await?.text().await?;
        let doc = dom_query::Document::from(body.clone());

        let meta = [
            doc.select(".toggle").iter().collect::<Vec<_>>().as_slice(),
            doc.select(".toggleVisible")
                .iter()
                .collect::<Vec<_>>()
                .as_slice(),
        ]
        .concat()
        .iter()
        .filter(|e| e.has_attr("id"))
        .filter(|e| {
            //
            let val = e.attr("id").unwrap();
            let res = (val.contains("rc")) || (val.contains("beta"));
            !res
        })
        .map(|e| {
            let version = e.attr("id").unwrap().to_string().trim().replace("go", "");
            let pack_list: Vec<PackageInfo> = e
                .select("tbody")
                .select("tr")
                .iter()
                .map(|item| {
                    let file_name = item.select("td").select("a").text().to_string();
                    let path = item
                        .select("td")
                        .select("a")
                        .attr("href")
                        .unwrap_or_default()
                        .to_string();
                    let checksum = item.select("td").select("tt").text().to_string();
                    PackageInfo::new(file_name, path, checksum)
                })
                .collect();
            VersionMeta::new(&version, pack_list)
        })
        .collect();

        Ok(meta)
    }
}

impl CommandAction {
    async fn local_version() -> anyhow::Result<HashSet<String>> {
        let mut local_version = HashSet::new();
        let home = Path::new(&ffg_home.clone()).join("packages");
        if !home.exists() {
            std::fs::create_dir_all(&home)?;
        }
        let dirs = fs::read_dir(home)?;
        dirs.for_each(|f| {
            let f = f.unwrap();
            let path = f.path();
            if !path.is_dir() {
                return;
            }
            let file_name = path.file_name();
            let file_name = file_name.unwrap().to_string_lossy();
            if !file_name.contains("go") {
                return;
            }
            local_version.insert(file_name.replace("go", ""));
        });
        Ok(local_version)
    }
}

#[derive(Debug, Clone)]
struct PackageInfo {
    path: String,
    file_name: String,
    sha256_checksum: String,
}

#[derive(Clone)]
struct VersionMeta {
    version: String,
    package: Vec<PackageInfo>,
}

impl VersionMeta {
    pub fn new(version: &str, pks: Vec<PackageInfo>) -> Self {
        Self {
            version: version.to_owned(),
            package: pks,
        }
    }
}

impl PackageInfo {
    pub fn new(file_name: String, path: String, sha256: String) -> Self {
        Self {
            file_name,
            path,
            sha256_checksum: sha256,
        }
    }
}

struct CommandUtil {
    // todo
}

impl CommandUtil {
    pub fn file_name(version: &str) -> String {
        let os = CommandUtil::get_os();
        let arch = CommandUtil::get_arch();
        let suffix = CommandUtil::get_suffix();
        format!("go{version}.{os}-{arch}.{suffix}")
    }
}

impl CommandUtil {
    pub async fn download(url: &str, save_path: &str) -> anyhow::Result<()> {
        let packages = Path::new(&preset::ffg_home.clone()).join("packages");
        if !packages.exists() {
            fs::create_dir_all(&packages)?;
        }
        let full_path = Path::new(save_path);

        if full_path.exists() {
            std::fs::remove_file(full_path)?;
        }

        let mut file = File::create(&full_path).await?;
        let response = reqwest::get(url).await?;
        let len = response.content_length().unwrap_or(0);
        let pb = ProgressBar::new(len);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>5}/{len:5} {msg}")?
                .progress_chars("=> "),
        );
        let mut stream = response.bytes_stream();
        while let Some(Ok(item)) = stream.next().await {
            let chunk_size = item.len();
            pb.inc(chunk_size.try_into().unwrap_or(0));
            file.write_all(&item).await?;
        }
        pb.finish();

        Ok(())
    }
}

impl CommandUtil {
    pub async fn sum_sha256(path: &str) -> anyhow::Result<String> {
        let input = Path::new(path);
        let val = try_async_digest(input).await?;
        Ok(val)
    }
}

impl CommandUtil {
    async fn extract_zip_async(src_path: &str, dst_path: &str) -> anyhow::Result<()> {
        let archive = File::open(src_path)
            .await
            .with_context(|| format!("failed to open zip {} error ", src_path.to_string().red()))?;
        let out_dir = Path::new(dst_path);
        CommandUtil::unzip_file(archive, out_dir).await?;
        Ok(())
    }
}

impl CommandUtil {
    async fn unpack_file(path: &str) -> anyhow::Result<()> {
        let dst_path = Path::new(&ffg_home.clone())
            .join(preset::pkgs.clone())
            .join("go");
        if dst_path.exists() {
            std::fs::remove_dir_all(dst_path)?;
        }
        let suffix = CommandUtil::get_suffix();
        if suffix == "tar.gz" {
            let tar_gz = std::fs::File::open(path)?;
            let tar = GzDecoder::new(tar_gz);
            let mut archive = Archive::new(tar);
            let unpack_path = Path::new(&preset::ffg_home.clone()).join(preset::pkgs.clone());
            archive.unpack(unpack_path)?;
        } else {
            // todo
            let dst_path = Path::new(&ffg_home.clone()).join(preset::pkgs.clone());
            CommandUtil::extract_zip_async(path, &dst_path.to_string_lossy()).await?;
        }
        Ok(())
    }
}

impl CommandUtil {
    pub fn get_os() -> &'static str {
        #[allow(unused_assignments)]
        let mut o = "<unknown>";
        let _ = o;
        #[cfg(target_os = "windows")]
        {
            o = "windows";
        }

        #[cfg(target_os = "linux")]
        {
            o = "linux";
        }

        #[cfg(target_os = "macos")]
        {
            o = "darwin";
        }
        o
    }
}

impl CommandUtil {
    pub fn get_suffix() -> &'static str {
        #[allow(unused_assignments)]
        let mut suffix: &str = "<unknown>";
        #[cfg(target_os = "windows")]
        {
            suffix = "zip";
        }
        #[cfg(not(target_os = "windows"))]
        {
            suffix = "tar.gz";
        }
        suffix
    }
}

impl CommandUtil {
    pub fn get_arch() -> &'static str {
        if cfg!(target_arch = "x86") {
            "386"
        } else if cfg!(target_arch = "x86_64") {
            "amd64"
        } else if cfg!(target_arch = "mips") {
            "mips"
        } else if cfg!(target_arch = "arm") {
            "arm"
        } else if cfg!(target_arch = "aarch64") {
            "arm64"
        } else {
            "<unknown>"
        }
    }
}

impl CommandUtil {
    /// Returns a relative path without reserved names, redundant separators, ".", or "..".
    pub fn sanitize_file_path(path: &str) -> PathBuf {
        // Replaces backwards slashes
        path.replace('\\', "/")
            // Sanitizes each component
            .split('/')
            .map(sanitize_filename::sanitize)
            .collect()
    }
}

impl CommandUtil {
    pub async fn unzip_file(archive_file: File, out_dir: &Path) -> anyhow::Result<()> {
        let archive = BufReader::new(archive_file).compat();
        let mut reader = ZipFileReader::new(archive)
            .await
            .expect("Failed to read zip file");
        for index in 0..reader.file().entries().len() {
            let entry = reader.file().entries().get(index).unwrap();
            let path = out_dir.join(CommandUtil::sanitize_file_path(
                entry.filename().as_str().unwrap(),
            ));
            // If the filename of the entry ends with '/', it is treated as a directory.
            // This is implemented by previous versions of this crate and the Python Standard Library.
            // https://docs.rs/async_zip/0.0.8/src/async_zip/read/mod.rs.html#63-65
            // https://github.com/python/cpython/blob/820ef62833bd2d84a141adedd9a05998595d6b6d/Lib/zipfile.py#L528
            let entry_is_dir = entry.dir().unwrap();

            let mut entry_reader = reader
                .reader_without_entry(index)
                .await
                .expect("Failed to read ZipEntry");

            if entry_is_dir {
                // The directory may have been created if iteration is out of order.
                if !path.exists() {
                    create_dir_all(&path)
                        .await
                        .expect("Failed to create extracted directory");
                }
            } else {
                // Creates parent directories. They may not exist if iteration is out of order
                // or the archive does not contain directory entries.
                let parent = path
                    .parent()
                    .expect("A file entry should have parent directories");
                if !parent.is_dir() {
                    create_dir_all(parent)
                        .await
                        .expect("Failed to create parent directories");
                }
                let writer = OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&path)
                    .await
                    .expect("Failed to create extracted file");
                futures_lite::io::copy(&mut entry_reader, &mut writer.compat_write())
                    .await
                    .expect("Failed to copy to extracted file");

                // Closes the file and manipulates its metadata here if you wish to preserve its metadata from the archive.
            }
        }
        Ok(())
    }
}

mod test {
    #[allow(unused_imports)]
    use super::CommandUtil;

    #[test]
    fn test_suffix() {
        let suffix = CommandUtil::get_suffix();
        #[cfg(target_os = "windows")]
        assert_eq!(suffix, "zip");

        #[cfg(not(target_os = "windows"))]
        assert_eq!(suffix, "tar.gz");
    }
}
