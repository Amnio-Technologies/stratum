use std::{collections::HashSet, env, fs, path::PathBuf, process::Command};

/// C macro used to tag exported functions from the plugin interface
const EXPORT_TAG: &str = "UI_EXPORT";

/// Constants from the C header we want to expose in Rust
const LVGL_BIND_VARS: &[&str] = &["LVGL_SCREEN_WIDTH", "LVGL_SCREEN_HEIGHT"];

fn get_dynamic_lib_path(root: &PathBuf, kind: &str) -> PathBuf {
    let ext = match env::var("CARGO_CFG_TARGET_OS").unwrap().as_str() {
        "windows" => "dll",
        "macos" => "dylib",
        _ => "so",
    };

    root.join("stratum-ui")
        .join("build")
        .join(kind)
        .join(format!("libstratum-ui.{ext}"))
}

fn build_dynamic_library(manifest: &PathBuf) {
    #[cfg(feature = "firmware")]
    let kind = "firmware";
    #[cfg(not(feature = "firmware"))]
    let kind = "desktop";

    let root = manifest.parent().unwrap().parent().unwrap();
    let lib = get_dynamic_lib_path(&root.to_path_buf(), kind);
    let script = root.join("stratum-ui").join("build.py");

    let status = Command::new("python3")
        .arg(&script)
        .arg("--dynamic")
        .status()
        .expect("Failed to run build.py --dynamic");

    if !status.success() {
        panic!(
            "build.py --dynamic failed with exit code {:?}",
            status.code()
        );
    }

    if !lib.exists() {
        panic!("Dynamic lib not found after build: {}", lib.display());
    }

    println!("cargo:rerun-if-changed={}", script.display());
    println!("cargo:rerun-if-changed={}", lib.display());
}

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let bindings_src_dir = manifest_dir.join("bindings");
    let bindings_src_file = bindings_src_dir.join("bindings.rs");
    let bindings_out_file = out_dir.join("bindings.rs");

    if is_native_build() {
        let (inc_dir_amnio, inc_dir_lvgl, header_to_bind) = locate_include_dirs(&manifest_dir);
        let fallback = "/usr/lib/clang/15/include";

        let (_api_funcs, allow_funcs, allow_types) = parse_amnio_api(&header_to_bind);

        run_bindgen(
            &header_to_bind,
            &inc_dir_amnio,
            &inc_dir_lvgl,
            fallback,
            &allow_funcs,
            &allow_types,
            &bindings_out_file,
        );

        commit_bindings(&bindings_src_dir, &bindings_src_file, &bindings_out_file);

        generate_dynamic_api(&bindings_out_file, &out_dir);
        generate_internal_api(&bindings_out_file, &out_dir);
    } else {
        copy_prebuilt_bindings(&bindings_src_file, &bindings_out_file);
    }

    link_static_library(&manifest_dir);
    build_dynamic_library(&manifest_dir);
}

// Detect native vs cross-compile
fn is_native_build() -> bool {
    env::var("HOST").unwrap() == env::var("TARGET").unwrap()
}

fn locate_include_dirs(manifest: &PathBuf) -> (PathBuf, PathBuf, PathBuf) {
    let root = manifest.parent().unwrap().parent().unwrap();
    let inc_amnio = root.join("stratum-ui").join("include");
    let inc_lvgl = inc_amnio.join("lvgl");
    let header = inc_amnio.join("stratum_ui.h");
    (inc_amnio, inc_lvgl, header)
}

fn parse_amnio_api(
    header: &PathBuf,
) -> (
    Vec<(String, Vec<(String, String)>)>,
    HashSet<String>,
    HashSet<String>,
) {
    use regex::Regex;
    let text = fs::read_to_string(header)
        .unwrap_or_else(|_| panic!("Failed to read header: {}", header.display()));

    let re = Regex::new(
        format!(
            r"{}\s+[^\s\(]+(?:\s*\*+)?\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\(([^)]*)\)\s*;",
            EXPORT_TAG
        )
        .as_str(),
    )
    .unwrap();

    let mut api = Vec::new();
    let mut funcs = HashSet::new();
    let mut types = HashSet::new();

    for cap in re.captures_iter(&text) {
        let name = cap[1].to_string();
        funcs.insert(name.clone());

        let raw = cap[2].trim();
        let mut args = Vec::new();

        if raw != "void" && !raw.is_empty() {
            for p in raw.split(',') {
                // tokenise on whitespace
                let mut toks: Vec<_> = p.trim().split_whitespace().collect();
                if toks.is_empty() {
                    continue;
                }

                // last token is the identifier (+ maybe leading '*'s)
                let mut var = toks.pop().unwrap().to_string();

                // collect pointer stars
                let mut ptr = String::new();

                // 1) stars glued to the identifier
                while var.starts_with('*') {
                    ptr.push('*');
                    var.remove(0);
                }

                // 2) stand-alone “*” tokens immediately before the identifier
                while toks.last().map_or(false, |t| *t == "*") {
                    toks.pop();
                    ptr.push('*');
                }

                // rebuild type and append the stars
                let mut ty = toks.join(" ");
                ty.push_str(&ptr);

                if !ty.is_empty() && !var.is_empty() {
                    types.insert(ty.clone());
                    args.push((ty, var));
                }
            }
        }
        api.push((name, args));
    }

    (api, funcs, types)
}

fn run_bindgen(
    header: &PathBuf,
    inc_amnio: &PathBuf,
    inc_lvgl: &PathBuf,
    fallback: &str,
    funcs: &HashSet<String>,
    types: &HashSet<String>,
    out_file: &PathBuf,
) {
    let mut b = bindgen::Builder::default()
        .layout_tests(false)
        .derive_default(false)
        .header(header.to_string_lossy())
        .clang_args(&[
            format!("-I{}", inc_amnio.display()),
            format!("-I{}", inc_lvgl.display()),
            format!("-isystem{}", fallback),
        ])
        .raw_line("#[allow(dead_code)]");

    for f in funcs {
        b = b.allowlist_function(f);
    }
    for t in types {
        b = b.allowlist_type(t);
    }

    for v in LVGL_BIND_VARS {
        b = b.allowlist_var(v);
    }

    let bindings = b.generate().expect("Failed to generate bindings");

    bindings
        .write_to_file(out_file)
        .expect("Couldn't write bindings.rs");
}

fn commit_bindings(src_dir: &PathBuf, src: &PathBuf, out: &PathBuf) {
    fs::create_dir_all(src_dir).unwrap();
    fs::copy(out, src).expect("Failed to update committed bindings.rs");
    println!("cargo:rerun-if-changed={}", src.display());
}

fn generate_dynamic_api(bindings: &PathBuf, out_dir: &PathBuf) {
    use proc_macro2::TokenStream;
    use quote::quote;
    use syn::{File, FnArg, ForeignItem, Item, ItemForeignMod};

    let src = fs::read_to_string(bindings).unwrap();
    let parsed: File = syn::parse_str(&src).unwrap();

    let mut out = String::new();
    out.push_str("use crate::stratum_ui_ffi::dynamic_api::internal_api::API;\n\n");

    for item in parsed.items {
        if let Item::ForeignMod(ItemForeignMod { items, .. }) = item {
            for fi in items {
                if let ForeignItem::Fn(func) = fi {
                    let sig = &func.sig;
                    let name = &sig.ident;
                    let args = sig
                        .inputs
                        .iter()
                        .filter_map(|arg| {
                            if let FnArg::Typed(p) = arg {
                                if let syn::Pat::Ident(id) = &*p.pat {
                                    let i = &id.ident;
                                    return Some(quote! {#i});
                                }
                            }
                            None
                        })
                        .collect::<Vec<TokenStream>>();

                    let code = quote! {
                        pub unsafe #sig {
                            (API.read().unwrap().as_ref().unwrap().api.#name)(#(#args),*)
                        }
                    };
                    out.push_str(&code.to_string());
                    out.push_str("\n\n");
                }
            }
        }
    }

    let dst = out_dir.join("dynamic_api.rs");
    fs::write(dst, out).expect("Failed to write dynamic_api.rs");
}

fn generate_internal_api(bindings: &PathBuf, out_dir: &PathBuf) {
    use quote::quote;
    use syn::{File, FnArg, ForeignItem, Item, ItemForeignMod, ReturnType};

    let src = fs::read_to_string(bindings).unwrap();
    let parsed: File = syn::parse_str(&src).unwrap();

    let mut api_fields = Vec::new();
    let mut api_loads = Vec::new();

    for item in parsed.items {
        if let Item::ForeignMod(ItemForeignMod { items, .. }) = item {
            for item in items {
                if let ForeignItem::Fn(func) = item {
                    let name = &func.sig.ident;

                    let mut arg_types = Vec::new();
                    for input in &func.sig.inputs {
                        if let FnArg::Typed(pat_type) = input {
                            let ty = &*pat_type.ty;
                            arg_types.push(quote! { #ty });
                        }
                    }

                    let ret_type = match &func.sig.output {
                        ReturnType::Default => quote! {},
                        ReturnType::Type(_, ty) => quote! { -> #ty },
                    };

                    let fn_type = quote! {
                        unsafe extern "C" fn(#(#arg_types),*) #ret_type
                    };

                    api_fields.push(quote! {
                        pub #name: #fn_type,
                    });

                    let symbol = format!("{}\0", name);
                    api_loads.push(quote! {
                        #name: *lib.get::<#fn_type>(#symbol.as_bytes()).unwrap(),
                    });
                }
            }
        }
    }

    let code = quote! {
        use std::sync::RwLock;
        use libloading::Library;
        use std::ffi::OsStr;
        use crate::stratum_ui_ffi::*;

        pub static API: RwLock<Option<LoadedApi>> = RwLock::new(None);

        pub struct LoadedApi {
            _lib: Library, // must be retained to keep function pointers valid
            pub api: Api,
        }

        pub struct Api {
            #(#api_fields)*
        }

        pub unsafe fn init_dynamic_bindings<P: AsRef<OsStr>>(lib_path: P) -> Result<(), String> {
            let lib = Library::new(&lib_path)
                .map_err(|e| format!("Failed to load dynamic lib: {e}"))?;

            let api = Api {
                #(#api_loads)*
            };

            *API.write().unwrap() = Some(LoadedApi {
                _lib: lib,
                api
            });

            Ok(())
        }
    };

    let dst = out_dir.join("internal_api.rs");
    fs::write(dst, code.to_string()).expect("Failed to write internal_api.rs");
}

fn copy_prebuilt_bindings(src: &PathBuf, dst: &PathBuf) {
    fs::create_dir_all(dst.parent().unwrap()).unwrap();
    fs::copy(src, dst).expect("Failed to copy pre-generated bindings.rs");
    println!("cargo:warning=Skipping bindgen (cross-compile)");
    println!("cargo:rerun-if-changed={}", src.display());
}

fn link_static_library(manifest: &PathBuf) {
    #[cfg(feature = "firmware")]
    let kind = "firmware";
    #[cfg(not(feature = "firmware"))]
    let kind = "desktop";

    let root = manifest.parent().unwrap().parent().unwrap();
    let build = root.join("stratum-ui").join("build").join(kind);
    let lib = build.join("libstratum-ui.a");

    if !lib.exists() {
        let script = root.join("stratum-ui").join("build.py");
        println!(
            "cargo:warning=libstratum-ui.a not found, running build script: {}",
            script.display()
        );

        let status = Command::new("python3")
            .arg(script)
            .status()
            .expect("Failed to run build.py");

        if !status.success() {
            panic!("build.py failed with exit code {:?}", status.code());
        }
    }

    if !lib.exists() {
        panic!("Static lib still not found after build: {}", lib.display());
    }

    println!("cargo:rustc-link-search=native={}", build.display());
    println!("cargo:rustc-link-lib=static=stratum-ui");
    println!("cargo:rerun-if-changed={}", lib.display());
}
