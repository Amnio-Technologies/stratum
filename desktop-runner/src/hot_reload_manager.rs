use chrono::Local;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    path::PathBuf,
    process::Command,
    sync::mpsc::{channel, Receiver, Sender},
    thread,
    time::{Duration, Instant},
};

pub struct HotReloadManager {
    plugin_path: PathBuf,
    build_script: PathBuf,
    watch_dirs: Vec<PathBuf>,
    debounce: Duration,
    tx_stop: Option<std::sync::mpsc::Sender<()>>, // Optional for graceful shutdown
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
        }
    }

    pub fn start(self) {
        let plugin_path = self.plugin_path.clone();
        let build_script = self.build_script.clone();
        let watch_dirs = self.watch_dirs.clone();

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

            let mut last_trigger = Instant::now();

            loop {
                match rx.recv() {
                    Ok(_) => {
                        let now = Instant::now();
                        if now.duration_since(last_trigger) >= self.debounce {
                            last_trigger = now;
                            println!("🔄 Change detected. Rebuilding...");

                            if let Err(e) = Self::rebuild_and_reload(&build_script, &plugin_path) {
                                eprintln!("❌ Reload failed: {e}");
                            } else {
                                println!("✅ Hot reload successful");
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        println!("👁️ Hot reload watcher started.");
    }

    /// Runs build.py --dynamic and reloads the plugin if successful
    pub fn rebuild_and_reload(
        build_script: &PathBuf,
        _plugin_path: &PathBuf,
    ) -> Result<(), String> {
        // determine extension
        let ext = if cfg!(target_os = "windows") {
            "dll"
        } else if cfg!(target_os = "macos") {
            "dylib"
        } else {
            "so"
        };

        // generate reload filename
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let stem = format!("stratum-ui_reload_{timestamp}");
        let filename = format!("lib{stem}.{ext}");

        // full path to built output
        let output_dir = PathBuf::from("../stratum-ui/build/desktop");
        let full_path = output_dir.join(&filename);

        // run build script with --output-name
        let status = Command::new("python3")
            .arg(build_script)
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

        // call init with the correct path
        unsafe {
            crate::amnio_bindings::init_dynamic_bindings(&full_path);
        }

        Ok(())
    }
}
