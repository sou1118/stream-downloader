use aes::{cipher::BlockDecryptMut, Aes128};
use anyhow::{anyhow, Context, Result};
use cbc::{
    cipher::{block_padding::Pkcs7, KeyIvInit},
    Decryptor,
};
use m3u8_rs::{parse_playlist_res, MediaPlaylist, Playlist};
use reqwest::get;
use serde_json::from_str;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;
use url::Url;

type Aes128CbcDec = Decryptor<Aes128>;

type Type = serde_json::Value;

#[tokio::main]
async fn main() -> Result<()> {
    let url = ""; // Set the URL here
    let response = get(url).await?.text().await?;
    let data: Type = from_str(&response)?;

    let episodes = data["episodes"].as_array().context("No episodes found")?;

    for episode in episodes {
        let title = episode["program_title"].as_str().unwrap_or("Unknown");
        let stream_url = episode["stream_url"]
            .as_str()
            .context("No stream URL found")?;

        println!("Downloading: {}", title);
        if let Err(e) = download_episode(title, stream_url).await {
            eprintln!("Error downloading episode {}: {}", title, e);
        }
    }

    Ok(())
}

async fn download_episode(title: &str, stream_url: &str) -> Result<()> {
    let output_dir = "downloads";
    create_dir_all(output_dir)?;

    let filename = format!("{}/{}.mp3", output_dir, title.replace(" ", "_"));
    let m3u8_content = get(stream_url).await?.text().await?;

    println!("M3U8 content:\n{}", m3u8_content);

    let playlist = parse_playlist_res(m3u8_content.as_bytes())
        .map_err(|e| anyhow!("Failed to parse m3u8: {:?}", e))?;

    match playlist {
        Playlist::MasterPlaylist(master) => {
            if let Some(variant) = master.variants.first() {
                let variant_url = Url::parse(stream_url)?.join(&variant.uri)?;
                download_variant(&filename, variant_url.as_str()).await?;
            } else {
                return Err(anyhow!("No variants found in master playlist"));
            }
        }
        Playlist::MediaPlaylist(media) => {
            download_media_playlist(&filename, stream_url, &media).await?;
        }
    }

    println!("Downloaded and converted: {}", filename);
    Ok(())
}

async fn download_media_playlist(
    output_file: &str,
    base_url: &str,
    media: &MediaPlaylist,
) -> Result<()> {
    let base_url = Url::parse(base_url)?;
    let key_url = media.segments[0]
        .key
        .as_ref()
        .context("No key found")?
        .uri
        .clone();
    let key_full_url = base_url.join(key_url.as_deref().context("Invalid key URL")?)?;
    let key = get(key_full_url).await?.bytes().await?;

    let segment_urls: Vec<String> = media
        .segments
        .iter()
        .map(|seg| base_url.join(&seg.uri).unwrap().to_string())
        .collect();

    let temp_file = NamedTempFile::new()?;
    let temp_path = temp_file.path().to_str().unwrap();

    let mut output = File::create(temp_path)?;

    for url in segment_urls {
        let resp = get(&url).await?.bytes().await?;
        let decrypted = decrypt_segment(&resp, &key)?;
        output.write_all(&decrypted)?;
    }

    let status = Command::new("ffmpeg")
        .args([
            "-f",
            "aac",
            "-i",
            temp_path,
            "-acodec",
            "libmp3lame",
            "-b:a",
            "128k",
            output_file,
        ])
        .status()?;

    if !status.success() {
        return Err(anyhow!("Failed to convert to MP3"));
    }

    Ok(())
}

fn decrypt_segment(encrypted: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    let iv = &encrypted[..16];
    let ciphertext = &encrypted[16..];

    let cipher = Aes128CbcDec::new_from_slices(key, iv)
        .map_err(|e| anyhow!("Failed to create cipher: {:?}", e))?;
    let mut buf = ciphertext.to_vec();
    let decrypted_data = cipher
        .decrypt_padded_mut::<Pkcs7>(&mut buf)
        .map_err(|e| anyhow!("Failed to decrypt: {:?}", e))?;

    Ok(decrypted_data.to_vec())
}

async fn download_variant(output_file: &str, variant_url: &str) -> Result<()> {
    let variant_content = get(variant_url).await?.text().await?;
    println!("Variant content:\n{}", variant_content);

    let playlist = parse_playlist_res(variant_content.as_bytes())
        .map_err(|e| anyhow!("Failed to parse variant m3u8: {:?}", e))?;

    match playlist {
        Playlist::MediaPlaylist(media) => {
            download_media_playlist(output_file, variant_url, &media).await?;
        }
        _ => return Err(anyhow!("Expected media playlist, found master playlist")),
    }

    Ok(())
}
