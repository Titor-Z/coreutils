// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::env;
use std::fs;
use std::path::Path;

use regex::Regex;

fn main() {
    generate_uutils_map();
    compile_ntsort();
    compile_manifest();
}

fn generate_uutils_map() {
    println!("cargo:rerun-if-changed=Cargo.toml");

    let manifest = fs::read_to_string("Cargo.toml").expect("failed to read Cargo.toml");
    let re = Regex::new(r#"^\s*(\w+)\s*=\s*\{.*package\s*=\s*"([^"]+)""#).unwrap();
    let mut coreutils = Vec::new();
    let mut entries = Vec::new();
    let mut has_findutils = false;
    let mut has_sort = false;
    let mut has_sed = false;
    let mut has_diffutils = false;
    let mut has_tar = false;

    for line in manifest.lines() {
        let Some(caps) = re.captures(line) else {
            continue;
        };
        let key = &caps[1];
        let package = &caps[2];

        if package == "findutils" {
            has_findutils = true;
        } else if package == "uu_sort" {
            has_sort = true;
        } else if package == "sed" {
            has_sed = true;
        } else if package == "diffutils" {
            has_diffutils = true;
        } else if package == "uu_tar" {
            has_tar = true;
        } else if let Some(util) = package.strip_prefix("uu_") {
            coreutils.push((util.to_string(), key.to_string()));
        }
    }

    for (util, alias) in &coreutils {
        let crate_ref = match alias.as_str() {
            "false" | "true" => format!("r#{alias}"),
            _ => alias.clone(),
        };
        let value = format!("({crate_ref}::uumain, {crate_ref}::uu_app)");

        if util == "test" {
            entries.push(("[".into(), value.clone()));
        }
        entries.push((util.clone(), value));
    }

    if has_findutils {
        entries.push(("find".into(), "(find_uumain, find_uu_app)".into()));
        entries.push(("xargs".into(), "(xargs_uumain, xargs_uu_app)".into()));
    }

    if has_sort {
        entries.push(("sort".into(), "(sort_uumain, sort_uu_app)".into()));
    }

    if has_sed {
        entries.push(("sed".into(), "(sed_uumain, sed_uu_app)".into()));
    }

    if has_diffutils {
        entries.push(("cmp".into(), "(cmp_uumain, cmp_uu_app)".into()));
        entries.push(("diff".into(), "(diff_uumain, diff_uu_app)".into()));
    }

    if has_tar {
        entries.push(("tar".into(), "(tar_uumain, tar_uu_app)".into()));
    }

    entries.push(("dfree".into(), "(dfree_uumain, dfree_uu_app)".into()));
    entries.push(("winfo".into(), "(winfo_uumain, winfo_uu_app)".into()));
    entries.push(("wtop".into(), "(wtop_uumain, wtop_uu_app)".into()));

    entries.sort();

    let mut phf_map = phf_codegen::OrderedMap::new();
    for (name, value) in &entries {
        phf_map.entry(name.as_str(), value.as_str());
    }

    let code = format!(
        "\
type UtilityMap<T> = phf::OrderedMap<&'static str, (fn(T) -> i32, fn() -> Command)>;

#[allow(clippy::too_many_lines)]
#[allow(clippy::unreadable_literal)]
fn util_map<T: Args>() -> UtilityMap<T> {{
{}
}}
",
        phf_map.build()
    );

    let out_dir = env::var("OUT_DIR").unwrap();
    fs::write(Path::new(&out_dir).join("uutils_map.rs"), code).unwrap();
}

fn compile_ntsort() {
    println!("cargo::rerun-if-changed=deps/ntsort/sort.c");
    println!("cargo::rerun-if-changed=deps/ntsort/sal_compat.h");

    let target = env::var("TARGET").unwrap_or_default();
    let is_msvc = target.contains("msvc");

    let mut build = cc::Build::new();
    build
        .file("deps/ntsort/sort.c")
        .define("NDEBUG", "1")
        .define("WIN32_LEAN_AND_MEAN", "1")
        .include("deps/ntsort");

    if is_msvc {
        build.define("UNICODE", "1");
        build.define("_UNICODE", "1");
    } else {
        // GCC/MinGW doesn't understand MSVC SAL annotations
        build.flag("-include").flag("sal_compat.h");
        // sort.c uses int where Win32 APIs expect DWORD; MSVC is lenient, GCC is strict
        build.flag("-Wno-incompatible-pointer-types");
        // GetTempPath2 is redefined to GetTempPathA since TCHAR=char without UNICODE
        build.define("GetTempPath2(cch,path)", "GetTempPathA(cch,path)");
    }

    build.compile("ntsort");
}

fn compile_manifest() {
    println!("cargo::rerun-if-changed=src/coreutils.manifest");

    winresource::WindowsResource::new()
        .set_manifest_file("src/coreutils.manifest")
        .set("FileDescription", "coreutils")
        .set(
            "LegalCopyright",
            "Copyright (c) uutils developers, Microsoft Corporation",
        )
        .set_icon("src/coreutils.ico")
        .compile()
        .unwrap();
}
