//! GitHub Releases update checker with semantic version comparison.
use serde::Serialize;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseCheck {
    pub status: String,
    pub current_version: String,
    pub latest_version: Option<String>,
    pub has_update: bool,
    pub release_url: String,
    pub release_name: Option<String>,
    pub published_at: Option<String>,
    pub notes: Option<String>,
}

/// Parses a semantic version like `0.0.1`, `v0.1.2`, or `v0.1.0-beta.1`.
/// Returns `(major, minor, patch, is_prerelease)`.
pub fn parse_semver(raw: &str) -> Option<(u64, u64, u64, bool)> {
    let trimmed = raw.trim().trim_start_matches(['v', 'V']);
    if trimmed.is_empty() {
        return None;
    }
    let without_build = trimmed.split('+').next()?;
    let (core, is_prerelease) = match without_build.split_once('-') {
        Some((head, pre)) if !pre.is_empty() => (head, true),
        _ => (without_build, false),
    };
    let mut parts = core.split('.');
    let major = parts.next()?.parse::<u64>().ok()?;
    let minor = parts.next()?.parse::<u64>().ok()?;
    let patch = parts.next().unwrap_or("0").parse::<u64>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((major, minor, patch, is_prerelease))
}

/// Returns `true` if `latest` is strictly newer than `current`.
pub fn is_newer(current: &str, latest: &str) -> bool {
    let Some((c_maj, c_min, c_pat, c_pre)) = parse_semver(current) else {
        return false;
    };
    let Some((l_maj, l_min, l_pat, l_pre)) = parse_semver(latest) else {
        return false;
    };
    match (l_maj, l_min, l_pat).cmp(&(c_maj, c_min, c_pat)) {
        std::cmp::Ordering::Greater => true,
        std::cmp::Ordering::Less => false,
        std::cmp::Ordering::Equal => c_pre && !l_pre,
    }
}

fn truncate_notes(raw: &str, max_chars: usize) -> Option<String> {
    let cleaned = raw.trim();
    if cleaned.is_empty() {
        return None;
    }
    let mut out = String::new();
    for (idx, ch) in cleaned.chars().enumerate() {
        if idx >= max_chars {
            out.push('…');
            break;
        }
        out.push(ch);
    }
    Some(out)
}

/// Checks GitHub Releases API (`GET /repos/{repo}/releases/latest`) for a newer release.
pub async fn check_github_release(repo: &str, current_version: &str) -> Result<ReleaseCheck, String> {
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(8))
        .timeout(Duration::from_secs(14))
        .user_agent(format!("Zwen/{current_version}"))
        .build()
        .map_err(|_| "无法初始化更新检查网络客户端".to_string())?;

    let api_url = format!("https://api.github.com/repos/{repo}/releases/latest");
    let releases_page = format!("https://github.com/{repo}/releases");

    let response = client
        .get(&api_url)
        .header(reqwest::header::ACCEPT, "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                "连接 GitHub 超时，请检查网络或代理设置".to_string()
            } else {
                "无法连接 GitHub，请检查网络后重试".to_string()
            }
        })?;

    let status_code = response.status();
    if status_code == reqwest::StatusCode::NOT_FOUND {
        return Ok(ReleaseCheck {
            status: "no_release".into(),
            current_version: current_version.into(),
            latest_version: None,
            has_update: false,
            release_url: releases_page,
            release_name: None,
            published_at: None,
            notes: None,
        });
    }
    if status_code == reqwest::StatusCode::FORBIDDEN || status_code == reqwest::StatusCode::TOO_MANY_REQUESTS {
        return Err("GitHub API 请求频率受限，请稍后再试".into());
    }
    if !status_code.is_success() {
        return Err(format!("GitHub 返回异常状态（HTTP {}）", status_code.as_u16()));
    }

    let payload: serde_json::Value = response
        .json()
        .await
        .map_err(|_| "解析 GitHub Release 数据失败".to_string())?;

    let tag = payload
        .get("tag_name")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "GitHub Release 缺少版本标签".to_string())?;

    let latest_clean = tag.trim_start_matches(['v', 'V']).to_string();
    let has_update = is_newer(current_version, tag);
    let release_url = payload
        .get("html_url")
        .and_then(|v| v.as_str())
        .filter(|s| s.starts_with("https://"))
        .unwrap_or(&releases_page)
        .to_string();
    let release_name = payload
        .get("name")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let published_at = payload
        .get("published_at")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let notes = payload
        .get("body")
        .and_then(|v| v.as_str())
        .and_then(|s| truncate_notes(s, 420));

    Ok(ReleaseCheck {
        status: if has_update {
            "update_available".into()
        } else {
            "up_to_date".into()
        },
        current_version: current_version.into(),
        latest_version: Some(latest_clean),
        has_update,
        release_url,
        release_name,
        published_at,
        notes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_standard_and_prefixed_semver() {
        assert_eq!(parse_semver("0.0.1"), Some((0, 0, 1, false)));
        assert_eq!(parse_semver("v0.0.2"), Some((0, 0, 2, false)));
        assert_eq!(parse_semver("V1.2.3-beta.1"), Some((1, 2, 3, true)));
        assert_eq!(parse_semver("0.2"), Some((0, 2, 0, false)));
        assert_eq!(parse_semver("not-a-version"), None);
    }

    #[test]
    fn compares_versions_accurately() {
        assert!(is_newer("0.0.1", "v0.0.2"));
        assert!(is_newer("0.0.1", "0.1.0"));
        assert!(is_newer("0.0.1-alpha.1", "0.0.1"));
        assert!(!is_newer("0.0.1", "v0.0.1"));
        assert!(!is_newer("0.0.2", "v0.0.1"));
        assert!(!is_newer("0.0.1", "0.0.1-rc.1"));
    }

    #[test]
    fn truncates_unicode_notes_safely() {
        assert_eq!(truncate_notes("  ", 10), None);
        assert_eq!(truncate_notes("更新日志内容", 4), Some("更新日志…".into()));
        assert_eq!(truncate_notes("short", 10), Some("short".into()));
    }
}
