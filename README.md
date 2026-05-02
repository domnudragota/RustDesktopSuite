# Rust Desktop Application Suite

A lightweight desktop companion app that shows **hourly weather** and **latest news**, extended with **Hadoop HDFS integration** for exporting and archiving per-user application data.

Built with **Rust** + **Slint**.

## Features

- **Weather (hourly, today):**
  - Current + next hours (temp, feels-like, precip chance, condition)
  - Auto day/night icons via `weather_codes.json`
  - Metric/Imperial units toggle (°C/°F)
  - Per-user caching and simple offline mode

- **News:**
  - Topic selector (e.g., *Top Stories*, *Trending*, *Sport*)
  - Tap an article to open it in your default browser
  - Per-user caching

- **Accounts:**
  - Start as `guest`
  - Register/login with a username + PIN
  - PINs are hashed with **Argon2** and stored locally in JSON
  - Quick account switching & deletion from the menu

- **Hadoop / HDFS Integration:**
  - Export per-user cached weather and news data into local JSON files
  - Upload exported files into **HDFS**
  - List archived HDFS files directly from the app
  - Store user-specific data under:
    - `/rust_desktop_suite/<username>/news_export.json`
    - `/rust_desktop_suite/<username>/weather_export.json`

## Screenshots

<img width="480" height="400" alt="image" src="https://github.com/user-attachments/assets/604ec649-73e2-4108-bda8-2a4afec7a9c1" />
<img width="480" height="400" alt="image" src="https://github.com/user-attachments/assets/8e0183e1-04f5-4a0e-8357-17b3dc01a952" />
<img width="480" height="400" alt="image" src="https://github.com/user-attachments/assets/58c61a3d-482b-4b2c-99f5-e65da269f985" />

## Project Structure

```text
src/
  main.rs           # App entrypoint, wiring, tasks, handlers
  auth.rs           # Local JSON-backed user store (Argon2 PIN hashing)
  cache.rs          # Simple per-user cache for weather/news
  config.rs         # Per-user settings (city, units, news topic)
  geocode.rs        # Geocoding via Open-Meteo geocoding API
  hadoop.rs         # Export to JSON + HDFS upload/list helpers
  news.rs           # News fetch logic (topic -> articles)
  weather.rs        # Weather fetcher + code→icon/description mapping
ui.slint            # Slint UI (pages, components)
weather_codes.json  # Weather code map (day/night label + icon URL)
icons/              # Static icons (e.g., cog)
icons_cache/        # Downloaded weather icons (created at runtime)
```

## How it Works

- **Weather**
  - Uses Open-Meteo APIs:
    - Geocoding: converts city name → latitude/longitude
    - Forecast: hourly temperature, apparent temperature, precipitation probability, weather code, `is_day`
  - `weather_codes.json` maps each `weather_code` to day/night descriptions and image URLs
  - Downloaded icons are cached in `icons_cache/`

- **News**
  - `news.rs` fetches a list of articles for the selected topic
  - News items are cached per user for offline fallback

- **Caching & Offline**
  - Weather/news responses are stored per user
  - On refresh, if cached data is fresh enough, the app can show cached content first
  - If online fetch fails, the app falls back to cached data when available

- **Settings**
  - City
  - Units (°C/°F)
  - News topic
  - Saved per user in JSON via `config.rs`

- **Hadoop / HDFS**
  - The app exports per-user cached data into local JSON files
  - These export files are then uploaded into **HDFS** using real `hadoop fs` commands executed from the Rust application
  - The app can also list files already stored in HDFS for the active user

## Hadoop Data Flow

1. The user refreshes **Weather** and **News**
2. The app stores the fetched results in local per-user cache
3. The user opens **Settings**
4. Pressing **Export Data** creates:
  - `weather_export.json`
  - `news_export.json`
5. Pressing **Upload to HDFS** runs Hadoop commands that:
  - create the user directory in HDFS
  - upload the exported JSON files
6. Pressing **List HDFS Files** shows the files currently stored in HDFS for that user

## Example HDFS Location

```text
/rust_desktop_suite/robica/news_export.json
/rust_desktop_suite/robica/weather_export.json
```

## Usage

1. Launch the app → you’re signed in as **guest** with default city/topic
2. Open **Settings** to change city, units, and topic; click **Save**
3. **Register** to create a local account (username + PIN)
4. Use the account menu (top right) to **switch users**, **log out**, or **delete** an account
5. On Weather/News pages, click **Refresh** to fetch latest data
6. In **Settings**, use:
  - **Export Data** to generate local JSON export files
  - **Upload to HDFS** to archive those files in Hadoop
  - **List HDFS Files** to inspect the files already stored for the current user

## Notes

- This project uses Hadoop primarily through **HDFS integration**
- Hadoop must be running before using the HDFS features
- The app expects the `hadoop` command to be available in the environment
- This is a desktop/demo project and not intended as a production distributed system