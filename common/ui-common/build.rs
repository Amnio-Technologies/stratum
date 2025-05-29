use std::error::Error;
use std::{collections::HashSet, env, fs, path::PathBuf, process::Command};

// TODO: add rerun-if-changed directives for UI code changes

use regex::Regex;

/// C macro used to tag exported functions from the plugin interface
const EXPORT_TAG: &str = "UI_EXPORT";

/// Constants from the C header we want to expose in Rust
const LVGL_BIND_VARS: &[&str] = &["LVGL_SCREEN_WIDTH", "LVGL_SCREEN_HEIGHT"];

fn main() -> Result<(), Box<dyn Error>> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let bindings_src_dir = manifest_dir.join("bindings");
    let bindings_src_file = bindings_src_dir.join("bindings.rs");
    let bindings_out_file = out_dir.join("bindings.rs");

    if is_cross_compile() {
        copy_prebuilt_bindings(&bindings_src_file, &bindings_out_file)?;
    } else {
        generate_bindings_via_bindgen(
            &manifest_dir,
            &out_dir,
            &bindings_src_dir,
            &bindings_src_file,
            &bindings_out_file,
        )?;
    }

    link_static_library(&manifest_dir)?;
    build_dynamic_library(&manifest_dir)?;

    Ok(())
}

fn copy_prebuilt_bindings(src: &PathBuf, dst: &PathBuf) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(dst.parent().unwrap())?;
    fs::copy(src, dst)?;
    println!("cargo:warning=Skipping bindgen (cross-compile)");
    println!("cargo:rerun-if-changed={}", src.display());

    Ok(())
}

fn generate_bindings_via_bindgen(
    manifest_dir: &PathBuf,
    out_dir: &PathBuf,
    bindings_src_dir: &PathBuf,
    bindings_src_file: &PathBuf,
    bindings_out_file: &PathBuf,
) -> Result<(), Box<dyn Error>> {
    let (inc_dir_amnio, inc_dir_lvgl, header_to_bind) = locate_include_dirs(&manifest_dir);
    // TODO remove this hard-coded clang path
    let fallback = "/mingw64/bin/clang";

    let (_api_funcs, allow_funcs, allow_types) =
        parse_lvscope_ffi_api(&header_to_bind, &inc_dir_amnio);

    run_bindgen(
        &header_to_bind,
        &inc_dir_amnio,
        &inc_dir_lvgl,
        fallback,
        &allow_funcs,
        &allow_types,
        &bindings_out_file,
    )?;

    commit_bindings(&bindings_src_dir, &bindings_src_file, &bindings_out_file)?;
    generate_dynamic_api(&bindings_out_file, &out_dir)?;
    generate_internal_api(&bindings_out_file, &out_dir)?;

    Ok(())
}

fn commit_bindings(src_dir: &PathBuf, src: &PathBuf, out: &PathBuf) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(src_dir)?;
    fs::copy(out, src)?;
    println!("cargo:rerun-if-changed={}", src.display());

    Ok(())
}

fn generate_dynamic_api(bindings: &PathBuf, out_dir: &PathBuf) -> Result<(), Box<dyn Error>> {
    use proc_macro2::TokenStream;
    use quote::quote;
    use syn::{File, FnArg, ForeignItem, Item, ItemForeignMod};

    let src = fs::read_to_string(bindings)?;
    let parsed: File = syn::parse_str(&src)?;

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
                                    return Some(quote! { #id });
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
    fs::write(dst, out)?;

    Ok(())
}

fn generate_internal_api(bindings: &PathBuf, out_dir: &PathBuf) -> Result<(), Box<dyn Error>> {
    use quote::quote;
    use syn::{File, FnArg, ForeignItem, Item, ItemForeignMod, ReturnType};

    let src = fs::read_to_string(bindings)?;
    let parsed: File = syn::parse_str(&src)?;

    let mut api_fields = Vec::new();
    let mut api_loads = Vec::new();

    for item in parsed.items {
        if let Item::ForeignMod(ItemForeignMod { items, .. }) = item {
            for fi in items {
                if let ForeignItem::Fn(func) = fi {
                    let name = &func.sig.ident;

                    let arg_types = func
                        .sig
                        .inputs
                        .iter()
                        .filter_map(|input| {
                            if let FnArg::Typed(pat_type) = input {
                                let ty = &*pat_type.ty;
                                return Some(quote! { #ty });
                            }
                            None
                        })
                        .collect::<Vec<_>>();

                    let ret_type = match &func.sig.output {
                        ReturnType::Default => quote! {},
                        ReturnType::Type(_, t) => quote! { -> #t },
                    };

                    let fn_type = quote! {
                        unsafe extern "C" fn(#(#arg_types),*) #ret_type
                    };

                    api_fields.push(quote! { pub #name: #fn_type, });

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
            _lib: Library,
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

            *API.write().unwrap() = Some(LoadedApi { _lib: lib, api });
            Ok(())
        }
    };

    let dst = out_dir.join("internal_api.rs");
    fs::write(dst, code.to_string())?;

    Ok(())
}

fn link_static_library(manifest: &PathBuf) -> Result<(), Box<dyn Error>> {
    let root = project_root(manifest);
    let kind_str = kind();
    let build_dir = root.join("stratum-ui").join("build").join(kind_str);
    let lib = build_dir.join("libstratum-ui.a");
    let script = root.join("stratum-ui").join("build.py");

    produce_artifact(&script, &[], &lib)?;
    println!("cargo:rustc-link-search=native={}", build_dir.display());
    println!("cargo:rustc-link-lib=static=stratum-ui");

    Ok(())
}

fn build_dynamic_library(manifest: &PathBuf) -> Result<(), Box<dyn Error>> {
    let root = project_root(manifest);
    let kind_str = kind();
    let lib = get_dynamic_lib_path(&root, kind_str);
    let script = root.join("stratum-ui").join("build.py");

    produce_artifact(&script, &["--dynamic"], &lib)?;

    Ok(())
}

fn produce_artifact(
    script: &PathBuf,
    script_args: &[&str],
    artifact: &PathBuf,
) -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::new("python3");
    cmd.arg(script);
    for arg in script_args {
        cmd.arg(arg);
    }

    let status = cmd
        .status()
        .map_err(|e| format!("failed to launch {:?}: {e}", script))?;
    if !status.success() {
        return Err(format!(
            "{:?} {:?} failed with exit code {:?}",
            script,
            script_args,
            status.code()
        )
        .into());
    }

    if !artifact.exists() {
        return Err(format!("Artifact not found after build: {}", artifact.display()).into());
    }

    println!("cargo:rerun-if-changed={}", script.display());

    Ok(())
}

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

fn kind() -> &'static str {
    #[cfg(feature = "firmware")]
    {
        "firmware"
    }
    #[cfg(not(feature = "firmware"))]
    {
        "desktop"
    }
}

/// climb up from manifest/ to the project root
fn project_root(manifest: &PathBuf) -> PathBuf {
    manifest.parent().unwrap().parent().unwrap().to_path_buf()
}

// Detect native vs cross-compile
fn is_cross_compile() -> bool {
    env::var("HOST").unwrap() != env::var("TARGET").unwrap()
}

fn locate_include_dirs(manifest: &PathBuf) -> (PathBuf, PathBuf, PathBuf) {
    let root = manifest.parent().unwrap().parent().unwrap();
    let inc_amnio = root.join("stratum-ui").join("include");
    let inc_lvgl = inc_amnio.join("lvgl");
    let header = inc_amnio.join("stratum_ui.h");
    (inc_amnio, inc_lvgl, header)
}

/// Recursively collect every header referenced by `header` via `#include "…"`
/// (only looking under `include_dir`), returning a flat Vec of PathBuf.
fn collect_headers(header: &PathBuf, include_dir: &PathBuf) -> Vec<PathBuf> {
    let mut visited = HashSet::new();
    let mut result = Vec::new();
    let inc_re = Regex::new(r#"#include\s*"([^"]+)""#).unwrap();

    fn recurse(
        path: &PathBuf,
        include_dir: &PathBuf,
        visited: &mut HashSet<PathBuf>,
        result: &mut Vec<PathBuf>,
        inc_re: &Regex,
    ) {
        if !visited.insert(path.clone()) {
            return;
        }
        result.push(path.clone());

        let text = fs::read_to_string(path)
            .unwrap_or_else(|_| panic!("Failed to read header: {}", path.display()));

        for cap in inc_re.captures_iter(&text) {
            let candidate = include_dir.join(&cap[1]);
            if candidate.exists() {
                recurse(&candidate, include_dir, visited, result, inc_re);
            }
        }
    }

    recurse(header, include_dir, &mut visited, &mut result, &inc_re);
    result
}

fn parse_lvscope_ffi_api(
    header: &PathBuf,
    include_dir: &PathBuf,
) -> (
    Vec<(String, Vec<(String, String)>)>,
    HashSet<String>,
    HashSet<String>,
) {
    let headers_to_parse = collect_headers(header, include_dir);
    parse_exported_apis(headers_to_parse)
}

fn extract_from_text(
    re: &Regex,
    header_text: &str,
    api: &mut Vec<(String, Vec<(String, String)>)>,
    funcs: &mut HashSet<String>,
    types: &mut HashSet<String>,
) {
    for cap in re.captures_iter(header_text) {
        let name = cap[1].to_string();
        funcs.insert(name.clone());

        let raw = cap[2].trim();
        let mut args = Vec::new();

        // Skip no-argument functions
        if raw == "void" || raw.is_empty() {
            api.push((name, args));
            continue;
        }

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

        api.push((name, args));
    }
}

fn parse_exported_apis(
    headers: Vec<PathBuf>,
) -> (
    Vec<(String, Vec<(String, String)>)>,
    HashSet<String>,
    HashSet<String>,
) {
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

    for header in headers {
        let header_text = fs::read_to_string(&header)
            .unwrap_or_else(|_| panic!("Failed to read header: {}", header.display()));
        extract_from_text(&re, &header_text, &mut api, &mut funcs, &mut types);
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
) -> Result<(), Box<dyn Error>> {
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

    let bindings = b.generate()?;
    bindings.write_to_file(out_file)?;

    Ok(())
}
