use chrono::Local;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
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
    tx_stop: Option<std::sync::mpsc::Sender<()>>,
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
            tx_stop: None,
            auto_reload: true,
            max_builds_to_keep: 5,
            reload_log: vec![],
            status: HotReloadStatus::Idle,
            last_reload_timestamp: "".into(),
            current_abi_hash: "".into(),
            should_reload_ui: AtomicBool::new(false),
        }
    }

    pub fn start(manager: SharedHotReloadManager) {
        let (watch_dirs, plugin_path, debounce) = {
            let guard = manager.lock().unwrap();
            (
                guard.watch_dirs.clone(),
                guard.plugin_path.clone(),
                guard.debounce,
            )
        };

        unsafe {
            stratum_ui_ffi::init_dynamic_bindings(plugin_path).unwrap();
        }

        Self::spawn_watch_thread(manager, watch_dirs, debounce);
        println!("👁️ Hot reload watcher started.");
    }

    fn spawn_watch_thread(
        manager: SharedHotReloadManager,
        watch_dirs: Vec<PathBuf>,
        debounce: Duration,
    ) {
        thread::spawn(move || {
            // now we *keep* watcher alive in this scope
            let (_watcher, _tx, rx) = Self::init_watcher(&watch_dirs);
            Self::watch_loop(manager, rx, debounce);
            // `_watcher` only drops here, after watch_loop exits
        });
    }

    fn init_watcher(watch_dirs: &[PathBuf]) -> (RecommendedWatcher, Sender<()>, Receiver<()>) {
        let (tx, rx) = channel();
        let transmit = tx.clone();
        let mut watcher = notify::recommended_watcher(move |res: Result<Event, _>| {
            if let Ok(event) = res {
                if !event.paths.is_empty() {
                    let _ = transmit.send(());
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

        loop {
            match rx.recv_timeout(Duration::from_millis(50)) {
                Ok(_) => last_event = Some(Instant::now()),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    if let Some(time) = last_event {
                        if time.elapsed() >= debounce {
                            last_event = None;
                            println!("🔄 Stable change detected. Rebuilding...");

                            let mut guard = manager.lock().unwrap();
                            guard.status = HotReloadStatus::Rebuilding;
                            if let Err(e) = guard.rebuild_and_reload() {
                                eprintln!("❌ Reload failed: {e}");
                            } else {
                                println!("✅ Hot reload successful");
                            }
                        }
                    }
                }
                Err(_) => break,
            }
        }
    }

    fn generate_reload_stem() -> String {
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        format!("stratum-ui_reload_{}", timestamp)
    }

    pub fn rebuild_and_reload(&mut self) -> Result<(), String> {
        let stem = Self::generate_reload_stem();
        let ext = Self::dylib_file_ext();
        let filename = format!("lib{}.{}", stem, ext);
        let full_path = PathBuf::from("../stratum-ui/build/desktop").join(&filename);

        // 👉 pass *just* the stem to Python:
        self.run_build_script(&stem)?;
        self.validate_plugin_exists(&full_path)?;

        self.load_build(&full_path);
        self.cull_old_builds();
        Ok(())
    }

    fn generate_reload_filename() -> String {
        let ext = Self::dylib_file_ext();
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        format!("libstratum-ui_reload_{}.{}", timestamp, ext)
    }

    fn run_build_script(&self, output_stem: &str) -> Result<(), String> {
        let status = Command::new("python3")
            .arg(&self.build_script)
            .arg("--dynamic")
            .arg("--output-name")
            .arg(output_stem)
            .status()
            .map_err(|e| format!("Build script failed to launch: {e}"))?;
        if !status.success() {
            Err(format!(
                "Build script failed with status: {:?}",
                status.code()
            ))
        } else {
            Ok(())
        }
    }

    fn validate_plugin_exists(&self, path: &Path) -> Result<(), String> {
        if !path.exists() {
            Err(format!("Built plugin not found at: {}", path.display()))
        } else {
            Ok(())
        }
    }

    pub fn load_build(&mut self, selected: &Path) {
        if !selected.exists() {
            self.status = HotReloadStatus::BuildFailed;
            return;
        }

        unsafe { stratum_ui_ffi::lvgl_teardown() }

        unsafe {
            match stratum_ui_ffi::init_dynamic_bindings(selected) {
                Ok(()) => {
                    self.plugin_path = selected.to_path_buf();
                    self.status = HotReloadStatus::ReloadSuccessful;
                    self.last_reload_timestamp = Self::now_string();
                    self.current_abi_hash = Self::generate_abi_hash(selected);
                    self.log_reload_result(true, selected);
                    self.should_reload_ui.store(true, Ordering::Relaxed);
                }
                Err(e) => {
                    self.status = HotReloadStatus::BuildFailed;
                    self.log_reload_result(false, selected);
                    self.reload_log.push(format!("❌ Error: {e}"));
                }
            }
        }
    }

    fn now_string() -> String {
        Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
    }

    fn generate_abi_hash(path: &Path) -> String {
        format!(
            "hash_{}",
            path.file_name()
                .unwrap_or_else(|| OsStr::new("unknown"))
                .to_string_lossy()
        )
    }

    fn log_reload_result(&mut self, success: bool, path: &Path) {
        let msg = if success {
            format!("✅ Manually loaded: {}", path.display())
        } else {
            format!("❌ Failed to load: {}", path.display())
        };
        self.reload_log.push(msg);
    }

    pub fn cull_old_builds(&self) {
        let mut builds = self.available_builds();
        builds.sort_by_key(|b| std::fs::metadata(&b.path).and_then(|m| m.modified()).ok());
        builds.reverse();

        for build in builds.iter().skip(self.max_builds_to_keep) {
            if !build.is_active {
                if let Err(e) = std::fs::remove_file(&build.path) {
                    eprintln!(
                        "⚠️ Failed to remove old build {}: {e}",
                        build.path.display()
                    );
                } else {
                    println!("🗑️ Removed old build: {}", build.path.display());
                }
            }
        }
    }

    pub fn available_builds(&self) -> Vec<BuildInfo> {
        let parent = self
            .plugin_path
            .parent()
            .expect("Hot reload plugin path should point to a file within a directory");

        let ext = Self::dylib_file_ext();
        let pattern = format!("libstratum-ui*.{ext}");

        let mut builds: Vec<_> = glob::glob(&format!("{}/{}", parent.display(), pattern))
            .unwrap()
            .filter_map(Result::ok)
            .collect();

        builds.sort_by_key(|p| std::fs::metadata(p).and_then(|m| m.modified()).ok());
        builds.reverse();

        builds
            .into_iter()
            .map(|path| {
                let is_active = path == self.plugin_path;
                BuildInfo { path, is_active }
            })
            .collect()
    }

    pub fn selected_build_display(&self) -> String {
        BuildInfo::new(self.plugin_path.clone(), true).filename()
    }

    fn dylib_file_ext() -> &'static str {
        if cfg!(target_os = "windows") {
            "dll"
        } else if cfg!(target_os = "macos") {
            "dylib"
        } else {
            "so"
        }
    }
}
