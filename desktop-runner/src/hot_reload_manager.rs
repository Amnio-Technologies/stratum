use chrono::Local;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use serde_json::json;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::{
    ffi::OsStr,
    path::PathBuf,
    process::Command,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};
use stratum_ui_common::stratum_ui_ffi;

// Compile-time plugin extension
const PLUGIN_EXT: &str = if cfg!(target_os = "windows") {
    "dll"
} else if cfg!(target_os = "macos") {
    "dylib"
} else {
    "so"
};

#[derive(Debug, Clone)]
pub enum HotReloadStatus {
    Idle,
    Rebuilding,
    BuildFailed,
    ReloadSuccessful,
    PluginMissing,
    LoadingPlugin,
    Unknown,
}

impl std::fmt::Display for HotReloadStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            HotReloadStatus::Idle => "✅ Idle",
            HotReloadStatus::Rebuilding => "🔄 Rebuilding...",
            HotReloadStatus::BuildFailed => "❌ Build Failed",
            HotReloadStatus::ReloadSuccessful => "✅ Last Reload Successful",
            HotReloadStatus::PluginMissing => "🚫 Plugin Missing",
            HotReloadStatus::LoadingPlugin => "🔃 Loading Plugin...",
            HotReloadStatus::Unknown => "❓ Unknown",
        };
        write!(f, "{msg}")
    }
}

pub struct HotReloadManager {
    plugin_path: PathBuf,
    build_script: PathBuf,
    watch_dirs: Vec<PathBuf>,
    debounce: Duration,
    pub auto_reload: bool,
    pub max_builds_to_keep: usize,
    pub reload_log: Vec<String>,
    pub status: HotReloadStatus,
    pub last_reload_timestamp: String,
    pub current_abi_hash: String,
    pub should_reload_ui: AtomicBool,
}

pub type SharedHotReloadManager = Arc<Mutex<HotReloadManager>>;

#[derive(Clone, Debug)]
pub struct BuildInfo {
    pub path: PathBuf,
    pub is_active: bool,
}

impl BuildInfo {
    pub fn new(path: PathBuf, is_active: bool) -> Self {
        Self { path, is_active }
    }

    pub fn filename(&self) -> String {
        self.path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into()
    }
}

impl HotReloadManager {
    pub fn new(
        plugin_path: PathBuf,
        build_script: PathBuf,
        watch_dirs: Vec<PathBuf>,
        debounce: Duration,
    ) -> Self {
        Self {
            plugin_path,
            build_script,
            watch_dirs,
            debounce,
            auto_reload: true,
            max_builds_to_keep: 5,
            reload_log: vec![],
            status: HotReloadStatus::Idle,
            last_reload_timestamp: String::new(),
            current_abi_hash: String::new(),
            should_reload_ui: AtomicBool::new(false),
        }
    }

    pub fn start(manager: SharedHotReloadManager) {
        let (plugin_path, watch_dirs, debounce) = {
            let guard = manager.lock().unwrap();
            (
                guard.plugin_path.clone(),
                guard.watch_dirs.clone(),
                guard.debounce,
            )
        };

        {
            let mut guard = manager.lock().unwrap();
            if let Err(e) = guard.install_plugin(&plugin_path) {
                guard
                    .reload_log
                    .push(format!("❌ Initial load failed: {e}"));
                guard.status = HotReloadStatus::BuildFailed;
            }
            guard
                .reload_log
                .push("👁️ Hot reload watcher started.".into());
        }

        Self::spawn_watch_thread(manager.clone(), watch_dirs, debounce);
    }

    fn spawn_watch_thread(
        manager: SharedHotReloadManager,
        watch_dirs: Vec<PathBuf>,
        debounce: Duration,
    ) {
        thread::spawn(move || {
            let (_watcher, _tx, rx) = Self::init_watcher(&watch_dirs);
            Self::watch_loop(manager, rx, debounce);
        });
    }

    fn init_watcher(watch_dirs: &[PathBuf]) -> (RecommendedWatcher, Sender<()>, Receiver<()>) {
        let (tx, rx) = channel();
        let tx_clone = tx.clone();
        let mut watcher = notify::recommended_watcher(move |res| {
            if let Ok(Event { paths, .. }) = res {
                if !paths.is_empty() {
                    let _ = tx_clone.send(());
                }
            }
        })
        .expect("Failed to create watcher");

        for dir in watch_dirs {
            watcher
                .watch(dir, RecursiveMode::Recursive)
                .unwrap_or_else(|_| panic!("Failed to watch {:?}", dir));
        }

        (watcher, tx, rx)
    }

    fn watch_loop(manager: SharedHotReloadManager, rx: Receiver<()>, debounce: Duration) {
        let mut last_event: Option<Instant> = None;
        const WATCHER_TICK_MS: u64 = 50;
        loop {
            match rx.recv_timeout(Duration::from_millis(WATCHER_TICK_MS)) {
                Ok(_) => {
                    last_event = Some(Instant::now());
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    if let Some(time) = last_event {
                        if time.elapsed() >= debounce {
                            last_event = None;
                            let mut guard = manager.lock().unwrap();
                            guard
                                .reload_log
                                .push("🔄 Stable change detected. Rebuilding...".into());
                            guard.status = HotReloadStatus::Rebuilding;
                            match guard.rebuild_and_reload() {
                                Err(e) => guard.reload_log.push(format!("❌ Reload failed: {e}")),
                                Ok(_) => guard.reload_log.push("✅ Hot reload successful".into()),
                            }
                        }
                    }
                }
                Err(_) => break,
            }
        }
    }

    fn install_plugin(&mut self, path: &Path) -> Result<(), String> {
        unsafe { stratum_ui_ffi::init_dynamic_bindings(path.to_path_buf()) }
            .map_err(|e| format!("Failed to install plugin: {e}"))
    }

    fn build_output_dir() -> PathBuf {
        PathBuf::from("../stratum-ui/build/desktop")
    }

    fn sorted_builds(&self) -> Vec<BuildInfo> {
        let mut infos: Vec<_> = glob::glob(&format!(
            "{}/libstratum-ui*.{}",
            Self::build_output_dir().display(),
            PLUGIN_EXT
        ))
        .unwrap()
        .filter_map(Result::ok)
        .map(|p| BuildInfo::new(p.clone(), p == self.plugin_path))
        .collect();
        infos.sort_by_key(|b| b.path.metadata().and_then(|m| m.modified()).ok());
        infos.reverse();
        infos
    }

    pub fn rebuild_and_reload(&mut self) -> Result<(), String> {
        let now = Local::now();
        let stem = format!("stratum-ui_reload_{}", now.format("%Y%m%d_%H%M%S"));
        let filename = format!("lib{}.{}", stem, PLUGIN_EXT);
        let full_path = Self::build_output_dir().join(&filename);

        self.run_build_script(&stem)?;
        self.validate_plugin_exists(&full_path)?;

        self.load_and_install(&full_path, now.clone());
        self.cull_old_builds();
        Ok(())
    }

    fn run_build_script(&self, output_stem: &str) -> Result<(), String> {
        // 1) Try the long-lived build_server daemon
        if let Ok(mut stream) = TcpStream::connect("127.0.0.1:9123") {
            // Build request
            let req = json!({
                "dynamic": true,
                "target": "desktop",
                "output_name": output_stem,
            })
            .to_string();
            // Send request
            stream
                .write_all(req.as_bytes())
                .map_err(|e| format!("Daemon write error: {}", e))?;
            let _ = stream.shutdown(std::net::Shutdown::Write);
            // Read response
            let mut buf = String::new();
            stream
                .read_to_string(&mut buf)
                .map_err(|e| format!("Daemon read error: {}", e))?;
            let resp: serde_json::Value = serde_json::from_str(&buf)
                .map_err(|e| format!("Invalid daemon response: {}", e))?;
            if resp.get("success").and_then(|v| v.as_bool()) == Some(true) {
                return Ok(());
            } else {
                return Err("Build daemon reported failure".into());
            }
        }

        // 2) Fallback to the original build.py invocation
        let status = Command::new("python3")
            .arg(&self.build_script)
            .arg("--dynamic")
            .arg("--output-name")
            .arg(output_stem)
            .status()
            .map_err(|e| format!("Build script failed to launch: {}", e))?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("Build script failed: {:?}", status.code()))
        }
    }

    fn validate_plugin_exists(&self, path: &Path) -> Result<(), String> {
        if path.exists() {
            Ok(())
        } else {
            Err(format!("Plugin not found: {}", path.display()))
        }
    }

    fn load_and_install(&mut self, path: &Path, now: chrono::DateTime<Local>) {
        if let Err(e) = self.install_plugin(path) {
            self.status = HotReloadStatus::BuildFailed;
            self.reload_log.push(format!("❌ Error: {e}"));
            return;
        }
        self.plugin_path = path.to_path_buf();
        self.status = HotReloadStatus::ReloadSuccessful;
        self.last_reload_timestamp = now.format("%Y-%m-%d %H:%M:%S").to_string();
        self.current_abi_hash = format!(
            "hash_{}",
            path.file_name()
                .unwrap_or_else(|| OsStr::new("unknown"))
                .to_string_lossy()
        );
        self.reload_log
            .push(format!("✅ Loaded: {}", path.display()));
        self.should_reload_ui.store(true, Ordering::Relaxed);
    }

    pub fn cull_old_builds(&mut self) {
        for build in self
            .sorted_builds()
            .into_iter()
            .skip(self.max_builds_to_keep)
        {
            if !build.is_active {
                if let Err(e) = std::fs::remove_file(&build.path) {
                    self.reload_log
                        .push(format!("⚠️ Failed to remove {}: {e}", build.path.display()));
                } else {
                    self.reload_log
                        .push(format!("🗑️ Removed old build: {}", build.path.display()));
                }
            }
        }
    }

    pub fn available_builds(&self) -> Vec<BuildInfo> {
        self.sorted_builds()
    }

    pub fn selected_build_display(&self) -> String {
        BuildInfo::new(self.plugin_path.clone(), true).filename()
    }

    pub fn load_plugin(&mut self, path: &Path) {
        let now = Local::now();
        self.load_and_install(path, now);
    }
}
