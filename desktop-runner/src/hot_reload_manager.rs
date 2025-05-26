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
    tx_stop: Option<std::sync::mpsc::Sender<()>>, // Optional for graceful shutdown
    pub auto_reload: bool,
    pub max_builds_to_keep: usize,
    pub reload_log: Vec<String>,
    pub status: HotReloadStatus,
    pub last_reload_timestamp: String,
    pub current_abi_hash: String,
    pub should_reload_ui: AtomicBool,
}

pub type SharedHotReloadManager = Arc<Mutex<HotReloadManager>>;

#[derive(Clone)]
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
            status: HotReloadStatus::Unknown,
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

        thread::spawn(move || {
            let (tx, rx): (Sender<()>, Receiver<()>) = channel();
            let mut watcher: RecommendedWatcher =
                notify::recommended_watcher(move |res: Result<Event, _>| {
                    if let Ok(event) = res {
                        if !event.paths.is_empty() {
                            let _ = tx.send(());
                        }
                    }
                })
                .expect("Failed to create watcher");

            for dir in &watch_dirs {
                watcher
                    .watch(dir, RecursiveMode::Recursive)
                    .unwrap_or_else(|_| panic!("Failed to watch {:?}", dir));
            }

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
        });

        println!("👁️ Hot reload watcher started.");
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

    /// Runs build.py --dynamic and reloads the plugin if successful
    pub fn rebuild_and_reload(&mut self) -> Result<(), String> {
        // determine extension
        let ext = Self::dylib_file_ext();

        // generate reload filename
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let stem = format!("stratum-ui_reload_{timestamp}");
        let filename = format!("lib{stem}.{ext}");

        // full path to built output
        let output_dir = PathBuf::from("../stratum-ui/build/desktop");
        let full_path = output_dir.join(&filename);

        // run build script with --output-name
        let status = Command::new("python3")
            .arg(self.build_script.clone())
            .arg("--dynamic")
            .arg("--output-name")
            .arg(&stem) // only stem, not full filename
            .status()
            .map_err(|e| format!("Build script failed to launch: {e}"))?;

        if !status.success() {
            return Err(format!(
                "Build script failed with status: {:?}",
                status.code()
            ));
        }

        if !full_path.exists() {
            return Err(format!(
                "Built plugin not found at: {}",
                full_path.display()
            ));
        }

        self.load_build(&full_path);

        Ok(())
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

    pub fn load_build(&mut self, selected: &Path) {
        if !selected.exists() {
            self.status = HotReloadStatus::BuildFailed;
            return;
        }

        unsafe {
            stratum_ui_ffi::lvgl_teardown();
        }

        unsafe {
            match crate::stratum_ui_ffi::init_dynamic_bindings(selected) {
                Ok(()) => {
                    self.plugin_path = selected.to_path_buf();

                    self.status = HotReloadStatus::ReloadSuccessful;
                    self.last_reload_timestamp =
                        chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

                    // TODO: compute real ABI hash
                    self.current_abi_hash = format!(
                        "hash_{}",
                        selected
                            .file_name()
                            .unwrap_or(OsStr::new("unknown"))
                            .to_string_lossy()
                    );

                    self.reload_log
                        .push(format!("✅ Manually loaded: {}", selected.display()));

                    self.should_reload_ui.store(true, Ordering::Relaxed);
                }
                Err(e) => {
                    self.status = HotReloadStatus::BuildFailed;
                    self.reload_log
                        .push(format!("❌ Failed to load {}: {e}", selected.display()));
                }
            }
        }
    }

    pub fn selected_build_display(&self) -> String {
        BuildInfo::new(self.plugin_path.clone(), true).filename()
    }
}
