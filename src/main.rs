use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::fs;
use std::fs::{DirEntry, File};
use std::io::{Read, Write};
use std::path::Path;
use clap::{Parser, Subcommand};
use serde_json::Value;

#[derive(Parser, Debug)]
#[command(author = "baoyun", version = "0.0.1", about = "塞壬唱片音乐下载器", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: SubCommands,
}

#[derive(Subcommand, Debug)]
enum SubCommands {
    /// 1. 初始化所有的专辑元数据
    InitAlbumsMeta {},
    /// 2. 创建每个专辑的文件夹和元数据文件
    CreateAlbumDirsAndMeta {},
    /// 3. 下载每个专辑的封面
    DownloadAlbumPics {},
    /// 4. 下载每个专辑的歌曲，wav格式
    DownloadAlbumSongs {},
}

type Result = std::result::Result<(), Box<dyn Error>>;

#[derive(Debug)]
struct MsmdError {
    message: String,
}

impl Display for MsmdError {
    fn fmt(&self, _: &mut Formatter<'_>) -> std::fmt::Result {
        println!("{}", self.message);
        Ok(())
    }
}

impl Error for MsmdError {}

#[tokio::main]
async fn main() -> Result {
    let args = Args::parse();

    match args.command {
        SubCommands::InitAlbumsMeta { .. } => {
            init_albums_meta().await?;
        }
        SubCommands::CreateAlbumDirsAndMeta { .. } => {
            create_album_dirs_and_meta().await?;
        }
        SubCommands::DownloadAlbumPics { .. } => {
            download_album_covers().await?;
        }
        SubCommands::DownloadAlbumSongs { .. } => {
            download_album_songs().await?;
        }
    }

    Ok(())
}

async fn download_album_songs() -> Result {
    let albums_dir = fs::read_dir("albums")?;
    for album_dir in albums_dir {
        let album_dir = album_dir?;
        let (name, path) = resolve_dir_names(&album_dir);
        if !album_dir.path().is_dir() {
            eprintln!("【{}】不是文件夹", name);
            continue;
        }

        let songs_done_path_str = format!("{}/.done_songs", &path);
        let songs_done_file = Path::new(&songs_done_path_str);

        if songs_done_file.exists() {
            println!("【{}】已下载歌曲，跳过", name);
            continue;
        }

        let mut album_meta = String::new();

        File::open(format!("{}/.meta.json", path))?
            .read_to_string(&mut album_meta)?;

        let album_meta: Value = serde_json::from_str(&album_meta)?;
        let songs = album_meta["songs"].as_array().unwrap();
        for song_meta in songs.iter() {
            let song_name = song_meta["name"].as_str().unwrap();
            let song_cid = song_meta["cid"].as_str().unwrap();

            let song_meta = reqwest::get(format!("https://monster-siren.hypergryph.com/api/song/{}", song_cid))
                .await?
                .json::<Value>()
                .await?;

            if song_meta["code"].as_i64().unwrap() == 0 {
                let song_url = song_meta["data"].as_object().unwrap()["sourceUrl"].as_str().unwrap();
                let song_bytes = reqwest::get(song_url)
                    .await?
                    .bytes()
                    .await?;
                File::create(format!("{}/{}.wav", path, song_name))?.write_all(&song_bytes)?;
            } else {
                eprintln!("【{}】加载歌曲元信息失败", song_name);
            }
        }
        File::create(songs_done_file)?.write("".as_bytes())?;
        println!("【{}】歌曲下载完成", name);
    }
    Ok(())
}

async fn download_album_covers() -> Result {
    let albums_dir = fs::read_dir("albums")?;

    for album_dir in albums_dir {
        let album_dir = album_dir?;
        let (name, path) = resolve_dir_names(&album_dir);
        if !album_dir.path().is_dir() {
            eprintln!("【{}】不是文件夹", name);
            continue;
        }

        let covers_done_path_str = format!("{}/.done_covers", &path);
        let covers_done_file = Path::new(&covers_done_path_str);

        if covers_done_file.exists() {
            println!("【{}】已下载封面，跳过", name);
            continue;
        }

        let mut album_meta = String::new();

        File::open(format!("{}/.meta.json", path))?
            .read_to_string(&mut album_meta)?;

        let album_meta: Value = serde_json::from_str(&album_meta)?;
        let cover_url = album_meta["coverUrl"].as_str().unwrap();
        let cover_de_url = album_meta["coverDeUrl"].as_str().unwrap();

        let cover = reqwest::get(cover_url);
        let cover_de = reqwest::get(cover_de_url);

        File::create(format!("{}/cover.jpg", path))?.write_all(&cover.await?.bytes().await?)?;
        File::create(format!("{}/cover_de.jpg", path))?.write_all(&cover_de.await?.bytes().await?)?;
        File::create(covers_done_file)?.write_all("".as_bytes())?;

        println!("【{}】封面下载完成", name);
    }
    Ok(())
}

fn resolve_dir_names(album_dir: &DirEntry) -> (String, String) {
    let name = String::from(album_dir.file_name().to_str().unwrap());
    let path = album_dir.path().display().to_string();
    (name, path)
}

async fn create_album_dirs_and_meta() -> Result {
    let mut albums_meta = String::new();

    let meta_path = Path::new(".albums.json");
    if meta_path.exists() && meta_path.is_file() {
        File::open(meta_path).unwrap().read_to_string(&mut albums_meta)?;
    } else {
        return Err(Box::new(MsmdError {
            message: "未找到专辑元数据文件, 请先执行 init_albums_meta".to_string()
        }));
    }

    let albums: Value = serde_json::from_str(&albums_meta)?;

    for album in albums.as_array()
        .unwrap()
        .iter() {
        let album_id = album["cid"].as_str().unwrap();
        let album_name = album["name"].as_str().unwrap();

        let album_path_str = format!("albums/{}", album_name);
        let album_dir = Path::new(&album_path_str);
        if !album_dir.exists() {
            fs::create_dir_all(album_dir)?;
        } else if !album_dir.is_dir() {
            eprintln!("【{}】已存在且不是文件夹", album_name);
            return Err(Box::new(MsmdError {
                message: String::new()
            }));
        }

        let album_done_file_path_str = format!("albums/{}/.done_album", album_name);
        let album_done_file = Path::new(&album_done_file_path_str);
        if album_done_file.exists() {
            println!("【{}】已加载完成，跳过", album_name);
            continue;
        }

        let album_meta_file_path_str = format!("albums/{}/.meta.json", album_name);
        let album_meta_file = Path::new(&album_meta_file_path_str);

        // 获取专辑详情
        let album_detail = reqwest::get(format!("https://monster-siren.hypergryph.com/api/album/{}/detail", album_id))
            .await?
            .json::<Value>()
            .await?;

        File::create(album_meta_file)?.write(album_detail["data"].to_string().as_bytes())?;
        File::create(album_done_file)?.write("".as_bytes())?;
        println!("【{}】初始化完成", album_name);
    }
    Ok(())
}

async fn init_albums_meta() -> Result {
    let albums = reqwest::get("https://monster-siren.hypergryph.com/api/albums")
        .await?
        .json::<Value>()
        .await?;
    File::create(".albums.json")?
        .write_all(albums["data"].to_string().as_bytes())
        .unwrap();
    Ok(())
}