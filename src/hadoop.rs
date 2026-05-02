use crate::cache::{load_news_for, load_weather_for};
use serde::Serialize;
use std::{env, fs, io, path::PathBuf, process::Command};

#[derive(Debug)]
pub enum HadoopError {
    Io(io::Error),
    Serde(serde_json::Error),
    MissingData(String),
    CommandFailed(String),
}

impl std::fmt::Display for HadoopError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HadoopError::Io(e) => write!(f, "I/O error: {}", e),
            HadoopError::Serde(e) => write!(f, "Serialization error: {}", e),
            HadoopError::MissingData(msg) => write!(f, "Missing data: {}", msg),
            HadoopError::CommandFailed(msg) => write!(f, "Hadoop command failed: {}", msg),
        }
    }
}

impl std::error::Error for HadoopError {}

impl From<io::Error> for HadoopError {
    fn from(e: io::Error) -> Self {
        HadoopError::Io(e)
    }
}

impl From<serde_json::Error> for HadoopError {
    fn from(e: serde_json::Error) -> Self {
        HadoopError::Serde(e)
    }
}

#[derive(Serialize)]
struct WeatherExportRow {
    user: String,
    city: String,
    units: String,
    cached_at: i64,
    time: String,
    temp: String,
    summary: String,
}

#[derive(Serialize)]
struct NewsExportRow {
    user: String,
    cached_at: i64,
    title: String,
    source: String,
    published: String,
    url: String,
}

fn app_exports_root() -> Result<PathBuf, HadoopError> {
    let home =
        env::var("HOME").map_err(|_| io::Error::new(io::ErrorKind::NotFound, "HOME not set"))?;
    let dir = PathBuf::from(home)
        .join("tock-workshop")
        .join("slint_rust")
        .join("exports");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn export_dir_for(user: &str) -> Result<PathBuf, HadoopError> {
    let dir = app_exports_root()?.join(user);
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn weather_export_path(user: &str) -> Result<PathBuf, HadoopError> {
    Ok(export_dir_for(user)?.join("weather_export.json"))
}

fn news_export_path(user: &str) -> Result<PathBuf, HadoopError> {
    Ok(export_dir_for(user)?.join("news_export.json"))
}

pub fn export_all_user_data(user: &str) -> Result<Vec<PathBuf>, HadoopError> {
    let mut exported_files = Vec::new();

    if let Some(weather_cache) = load_weather_for(user) {
        let rows: Vec<WeatherExportRow> = weather_cache
            .rows
            .into_iter()
            .map(|row| WeatherExportRow {
                user: user.to_string(),
                city: weather_cache.city.clone(),
                units: weather_cache.units.clone(),
                cached_at: weather_cache.ts,
                time: row.time,
                temp: row.temp,
                summary: row.summary,
            })
            .collect();

        let path = weather_export_path(user)?;
        let json = serde_json::to_string_pretty(&rows)?;
        fs::write(&path, json)?;
        exported_files.push(path);
    }

    if let Some(news_cache) = load_news_for(user) {
        let rows: Vec<NewsExportRow> = news_cache
            .rows
            .into_iter()
            .map(|row| NewsExportRow {
                user: user.to_string(),
                cached_at: news_cache.ts,
                title: row.title,
                source: row.source,
                published: row.published,
                url: row.url,
            })
            .collect();

        let path = news_export_path(user)?;
        let json = serde_json::to_string_pretty(&rows)?;
        fs::write(&path, json)?;
        exported_files.push(path);
    }

    if exported_files.is_empty() {
        return Err(HadoopError::MissingData(
            "No cached weather/news data found. Refresh weather or news first.".to_string(),
        ));
    }

    Ok(exported_files)
}

fn run_hadoop_command(args: &[String]) -> Result<String, HadoopError> {
    let output = Command::new("hadoop").args(args).output()?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if output.status.success() {
        Ok(stdout)
    } else {
        let msg = if !stderr.is_empty() { stderr } else { stdout };
        Err(HadoopError::CommandFailed(msg))
    }
}

pub fn upload_exports_to_hdfs(user: &str) -> Result<String, HadoopError> {
    let export_dir = export_dir_for(user)?;
    let weather_path = export_dir.join("weather_export.json");
    let news_path = export_dir.join("news_export.json");

    if !weather_path.exists() && !news_path.exists() {
        return Err(HadoopError::MissingData(
            "No export files found. Press 'Export Data' first.".to_string(),
        ));
    }

    let hdfs_dir = format!("/rust_desktop_suite/{user}");

    run_hadoop_command(&vec![
        "fs".to_string(),
        "-mkdir".to_string(),
        "-p".to_string(),
        hdfs_dir.clone(),
    ])?;

    let mut uploaded_names = Vec::new();

    if weather_path.exists() {
        run_hadoop_command(&vec![
            "fs".to_string(),
            "-put".to_string(),
            "-f".to_string(),
            weather_path.to_string_lossy().to_string(),
            hdfs_dir.clone(),
        ])?;
        uploaded_names.push("weather_export.json".to_string());
    }

    if news_path.exists() {
        run_hadoop_command(&vec![
            "fs".to_string(),
            "-put".to_string(),
            "-f".to_string(),
            news_path.to_string_lossy().to_string(),
            hdfs_dir.clone(),
        ])?;
        uploaded_names.push("news_export.json".to_string());
    }

    Ok(format!(
        "Uploaded to HDFS: {} -> {}",
        uploaded_names.join(", "),
        hdfs_dir
    ))
}

pub fn list_hdfs_user_files(user: &str) -> Result<String, HadoopError> {
    let hdfs_dir = format!("/rust_desktop_suite/{user}");

    let result = run_hadoop_command(&vec!["fs".to_string(), "-ls".to_string(), hdfs_dir.clone()])?;

    if result.trim().is_empty() {
        Ok(format!("No files found in {}", hdfs_dir))
    } else {
        Ok(result)
    }
}
