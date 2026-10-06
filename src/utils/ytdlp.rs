use std::time::Duration;

use serde::Deserialize;
use tracing::debug;

use crate::utils::track_utils::TrackMetadata;

/// Most songs a single playlist link will queue
pub const MAX_PLAYLIST_TRACKS: usize = 50;

/// What a URL turned out to be
#[derive(Debug)]
pub enum UrlInfo {
    Video(TrackMetadata),
    Playlist(Playlist),
}

#[derive(Debug)]
pub struct Playlist {
    pub title: Option<String>,
    /// (video URL, metadata) for each playable entry, in playlist order
    pub tracks: Vec<(String, TrackMetadata)>,
    /// Entries skipped because they're private, deleted, or otherwise unavailable
    pub unavailable: usize,
    /// The playlist had more than [`MAX_PLAYLIST_TRACKS`] entries
    pub truncated: bool,
}

#[derive(Deserialize)]
struct Thumbnail {
    url: String,
}

#[derive(Deserialize)]
struct Info {
    #[serde(rename = "_type")]
    kind: Option<String>,
    title: Option<String>,
    duration: Option<f64>,
    thumbnail: Option<String>,
    #[serde(default)]
    entries: Vec<Entry>,
}

#[derive(Deserialize)]
struct Entry {
    url: Option<String>,
    title: Option<String>,
    duration: Option<f64>,
    #[serde(default)]
    thumbnails: Vec<Thumbnail>,
}

/// Ask yt-dlp what a URL points at. Playlists are listed without resolving
/// each video (fast); a video link that's also in a playlist
/// (`watch?v=…&list=…`) counts as just the video.
pub async fn inspect_url(url: &str) -> Result<UrlInfo, String> {
    let output = tokio::process::Command::new("yt-dlp")
        .args([
            "--flat-playlist",
            "--no-playlist",
            "--dump-single-json",
            "--playlist-end",
            // One extra so we can tell the playlist was cut short
            &(MAX_PLAYLIST_TRACKS + 1).to_string(),
            // End of options, so a URL starting with "-" isn't parsed as a flag
            "--",
            url,
        ])
        .output()
        .await
        .map_err(|err| format!("failed to run yt-dlp: {err}"))?;

    if !output.status.success() {
        return Err(format!(
            "yt-dlp failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    parse_url_info(&output.stdout)
}

fn parse_url_info(json: &[u8]) -> Result<UrlInfo, String> {
    let info: Info =
        serde_json::from_slice(json).map_err(|err| format!("unexpected yt-dlp output: {err}"))?;

    if info.kind.as_deref() != Some("playlist") {
        return Ok(UrlInfo::Video(TrackMetadata {
            title: info
                .title
                .unwrap_or_else(|| String::from("Unknown Track Title")),
            thumbnail_url: info.thumbnail,
            duration: info.duration.map(Duration::from_secs_f64),
        }));
    }

    let truncated = info.entries.len() > MAX_PLAYLIST_TRACKS;
    let mut unavailable = 0;
    let mut tracks = Vec::new();

    for entry in info.entries.into_iter().take(MAX_PLAYLIST_TRACKS) {
        // yt-dlp leaves out the title of private/deleted videos, or uses a
        // placeholder like "[Private video]"
        let (Some(url), Some(title)) = (entry.url, entry.title) else {
            unavailable += 1;
            continue;
        };
        if title == "[Private video]" || title == "[Deleted video]" {
            unavailable += 1;
            continue;
        }

        tracks.push((
            url,
            TrackMetadata {
                title,
                // The last thumbnail is usually the highest quality
                thumbnail_url: entry.thumbnails.into_iter().last().map(|t| t.url),
                duration: entry.duration.map(Duration::from_secs_f64),
            },
        ));
    }

    debug!(
        "Playlist '{:?}': {} playable, {} unavailable, truncated: {}",
        info.title,
        tracks.len(),
        unavailable,
        truncated
    );

    Ok(UrlInfo::Playlist(Playlist {
        title: info.title,
        tracks,
        unavailable,
        truncated,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Trimmed from real `yt-dlp --flat-playlist --no-playlist -J` output

    const VIDEO: &str = r#"{
        "_type": "video",
        "id": "dQw4w9WgXcQ",
        "title": "Rick Astley - Never Gonna Give You Up (Official Video) (4K Remaster)",
        "duration": 213,
        "thumbnail": "https://i.ytimg.com/vi_webp/dQw4w9WgXcQ/maxresdefault.webp",
        "thumbnails": [{"url": "https://i.ytimg.com/vi/dQw4w9WgXcQ/default.jpg"}]
    }"#;

    const PLAYLIST: &str = r#"{
        "_type": "playlist",
        "title": "Lynyrd Skynyrd  Greatest Hits",
        "entries": [
            {"_type": "url", "id": "71qaX05-ArM", "url": "https://www.youtube.com/watch?v=71qaX05-ArM",
             "title": "Lynyrd Skynyrd-The Ballad of Curtis Loew", "duration": 292,
             "thumbnails": [{"url": "https://i.ytimg.com/vi/71qaX05-ArM/hqdefault.jpg?small"},
                            {"url": "https://i.ytimg.com/vi/71qaX05-ArM/hqdefault.jpg?large"}]},
            {"_type": "url", "id": "iFNbTdLfBwQ", "url": "https://www.youtube.com/watch?v=iFNbTdLfBwQ",
             "title": null, "duration": null, "thumbnails": []},
            {"_type": "url", "id": "aaaaaaaaaaa", "url": "https://www.youtube.com/watch?v=aaaaaaaaaaa",
             "title": "[Private video]", "duration": null},
            {"_type": "url", "id": "8eNoms9wsGc", "url": "https://www.youtube.com/watch?v=8eNoms9wsGc",
             "title": "Lynyrd Skynyrd - Simple Man (Audio)", "duration": 356.5}
        ]
    }"#;

    #[test]
    fn it_parses_a_single_video() {
        let UrlInfo::Video(metadata) = parse_url_info(VIDEO.as_bytes()).unwrap() else {
            panic!("expected a video");
        };

        assert_eq!(
            metadata.title,
            "Rick Astley - Never Gonna Give You Up (Official Video) (4K Remaster)"
        );
        assert_eq!(metadata.duration, Some(Duration::from_secs(213)));
        assert_eq!(
            metadata.thumbnail_url.as_deref(),
            Some("https://i.ytimg.com/vi_webp/dQw4w9WgXcQ/maxresdefault.webp")
        );
    }

    #[test]
    fn it_parses_a_playlist_and_skips_unavailable_entries() {
        let UrlInfo::Playlist(playlist) = parse_url_info(PLAYLIST.as_bytes()).unwrap() else {
            panic!("expected a playlist");
        };

        assert_eq!(
            playlist.title.as_deref(),
            Some("Lynyrd Skynyrd  Greatest Hits")
        );
        assert_eq!(playlist.unavailable, 2);
        assert!(!playlist.truncated);

        let titles: Vec<&str> = playlist
            .tracks
            .iter()
            .map(|(_, m)| m.title.as_str())
            .collect();
        assert_eq!(
            titles,
            [
                "Lynyrd Skynyrd-The Ballad of Curtis Loew",
                "Lynyrd Skynyrd - Simple Man (Audio)"
            ]
        );

        let (url, first) = &playlist.tracks[0];
        assert_eq!(url, "https://www.youtube.com/watch?v=71qaX05-ArM");
        assert_eq!(first.duration, Some(Duration::from_secs(292)));
        assert_eq!(
            first.thumbnail_url.as_deref(),
            Some("https://i.ytimg.com/vi/71qaX05-ArM/hqdefault.jpg?large")
        );
        assert_eq!(
            playlist.tracks[1].1.duration,
            Some(Duration::from_secs_f64(356.5))
        );
    }

    #[test]
    fn it_caps_long_playlists() {
        let entries: Vec<String> = (0..MAX_PLAYLIST_TRACKS + 1)
            .map(|i| {
                format!(r#"{{"url": "https://www.youtube.com/watch?v={i}", "title": "Song {i}"}}"#)
            })
            .collect();
        let json = format!(
            r#"{{"_type": "playlist", "entries": [{}]}}"#,
            entries.join(",")
        );

        let UrlInfo::Playlist(playlist) = parse_url_info(json.as_bytes()).unwrap() else {
            panic!("expected a playlist");
        };

        assert!(playlist.truncated);
        assert_eq!(playlist.tracks.len(), MAX_PLAYLIST_TRACKS);
        assert_eq!(playlist.tracks.last().unwrap().1.title, "Song 49");
    }

    #[test]
    fn it_rejects_output_that_isnt_json() {
        assert!(parse_url_info(b"ERROR: Unsupported URL").is_err());
    }
}
