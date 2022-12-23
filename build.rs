use std::{env, path::PathBuf};

// Fixes [753](https://github.com/rust-lang/rust-bindgen/issues/753) to generate _IOC macro
#[derive(Debug)]
pub struct Fix753 {}
impl bindgen::callbacks::ParseCallbacks for Fix753 {
    fn item_name(&self, original_item_name: &str) -> Option<String> {
        Some(original_item_name.trim_start_matches("Fix753_").to_owned())
    }
}

#[derive(Debug)]
pub struct AnonIov {}
impl bindgen::callbacks::ParseCallbacks for AnonIov {
    fn item_name(&self, _original_item_name: &str) -> Option<String> {
        if _original_item_name == "exmap_iov__bindgen_ty_1__bindgen_ty_1" {
            Some("ExmapIov".to_owned())
        } else {
            Some(_original_item_name.to_owned())
        }
    }
}

fn main() {
    const INCLUDE: &str = r#"
#include <linux/exmap.h>

#define MARK_FIX_753(req_name) const unsigned long int Fix753_##req_name = req_name;

MARK_FIX_753(EXMAP_IOCTL_ACTION);
MARK_FIX_753(EXMAP_IOCTL_SETUP);
    "#;

    #[cfg(not(feature = "overwrite"))]
    let outdir = PathBuf::from(env::var("OUT_DIR").unwrap());

    #[cfg(feature = "overwrite")]
    let outdir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("src/sys");

    let builder = bindgen::Builder::default().header_contents("include-file.h", INCLUDE);

    builder
        .derive_default(true)
        .generate_comments(true)
        .allowlist_var("EXMAP_.*|Fix753.*|__NR_ioctl")
        .allowlist_type("exmap_.*")
        .anon_fields_prefix("anon")
        .parse_callbacks(Box::new(Fix753 {}))
        .parse_callbacks(Box::new(AnonIov {}))
        .prepend_enum_name(false)
        .use_core()
        .generate()
        .unwrap()
        .write_to_file(outdir.join("sys.rs"))
        .unwrap();
}
