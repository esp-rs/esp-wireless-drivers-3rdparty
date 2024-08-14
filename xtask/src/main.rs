use core::str;
use std::{
    env, fs,
    path::{Path, PathBuf},
};

use clap::Parser;

#[derive(Debug, Parser)]
struct Cli {
    chips: Vec<String>,
}

// TODO avoid full paths in compiled code

fn main() {
    env_logger::init();
    let args = Cli::parse();

    if !std::env::var("IDF_PATH").is_ok() {
        eprintln!("No activated ESP-IDF installation");
        std::process::exit(-2);
    }

    if !Path::new("version").exists() {
        eprintln!("Execute in the root of the project");
        std::process::exit(-1);
    }

    let chips = if args.chips.is_empty() {
        vec![
            "esp32".to_string(),
            "esp32s2".to_string(),
            "esp32s3".to_string(),
            "esp32c2".to_string(),
            "esp32c3".to_string(),
            "esp32c6".to_string(),
            "esp32h2".to_string(),
        ]
    } else {
        args.chips
    };

    for chip in chips {
        process(&chip);
    }

    // copy chip independent headers
    let idf_path = std::env::var("IDF_PATH").unwrap();
    let dst = "./include";

    log::info!("Copy common headers");
    copy_files(
        &format!("{idf_path}/components/wpa_supplicant/esp_supplicant/include"),
        &format!("{dst}"),
    );
    copy_files(
        &format!("{idf_path}/components/esp_phy/include"),
        &format!("{dst}"),
    );
    copy_files(
        &format!("{idf_path}/components/esp_phy/include/esp_private"),
        &format!("{dst}"),
    );
    copy_files(
        &format!("{idf_path}/components/esp_coex/include"),
        &format!("{dst}"),
    );    
    copy_files(
        &format!("{idf_path}/components/esp_wifi/include"),
        &format!("{dst}"),
    );
    copy_files(
        &format!("{idf_path}/components/esp_wifi/include/esp_private"),
        &format!("{dst}/esp_private"),
    );
    copy_files(
        &format!("{idf_path}/components/esp_coex/include/private"),
        &format!("{dst}"),
    ); 
    copy_files(
        &format!("{idf_path}/components/esp_timer/include"),
        &format!("{dst}"),
    );
    copy_files(
        &format!("{idf_path}/components/esp_system/include"),
        &format!("{dst}"),
    );
    copy_files(
        &format!("{idf_path}/components/esp_event/include"),
        &format!("{dst}"),
    );
    copy_files(
        &format!("{idf_path}/components/nvs_flash/include"),
        &format!("{dst}"),
    );
    copy_file(
        &format!("{idf_path}/components/esp_common/include/esp_err.h"),
        &format!("{dst}/esp_err.h"),
    );
    copy_file(
        &format!("{idf_path}/components/esp_common/include/esp_compiler.h"),
        &format!("{dst}/esp_compiler.h"),
    );
    copy_file(
        &format!("{idf_path}/components/esp_hw_support/include/esp_interface.h"),
        &format!("{dst}/esp_interface.h"),
    );

    replace_in_file(&format!("{dst}/esp_coexist_internal.h"), "private/", "");
    replace_in_file(&format!("{dst}/esp_event.h"), r#"#include "freertos/FreeRTOS.h""#, r#""#);
    replace_in_file(&format!("{dst}/esp_event.h"), r#"#include "freertos/task.h""#, r#""#);
    replace_in_file(&format!("{dst}/esp_event.h"), r#"#include "freertos/queue.h""#, r#""#);
    replace_in_file(&format!("{dst}/esp_event.h"), r#"#include "freertos/semphr.h""#, r#""#);
    replace_in_file(&format!("{dst}/esp_system.h"), r#"#include "esp_attr.h""#, r#""#);
    replace_in_file(&format!("{dst}/esp_system.h"), r#"#include "esp_bit_defs.h""#, r#""#);
    replace_in_file(&format!("{dst}/esp_system.h"), r#"#include "esp_idf_version.h""#, r#""#);
    replace_in_file(&format!("{dst}/nvs.h"), r#"#include "esp_attr.h""#, r#""#);
    replace_in_file(&format!("{dst}/esp_private/esp_wifi_private.h"), r#"#include "freertos/FreeRTOS.h""#, r#""#);
    replace_in_file(&format!("{dst}/esp_private/esp_wifi_private.h"), r#"#include "freertos/queue.h""#, r#""#);
    replace_in_file(&format!("{dst}/esp_private/wifi.h"), r#"#include "freertos/FreeRTOS.h""#, r#""#);
    replace_in_file(&format!("{dst}/esp_private/wifi.h"), r#"#include "freertos/queue.h""#, r#""#);
}

fn process(chip: &str) {
    log::info!("Processing {chip}");

    // // clean
    log::info!("Clean");
    remove_dir_all("./helper_project/build");
    remove_file("./helper_project/sdkconfig");
    remove_file("./helper_project/sdkconfig.defaults");
    remove_file("./helper_project/sdkconfig.old");

    // build
    log::info!("Build");
    copy_file(
        &format!("./patch/{chip}/sdkconfig.defaults"),
        "./helper_project/sdkconfig.defaults",
    );

    build(
        "helper_project",
        &[&format!("-DIDF_TARGET={chip}"), "build"],
    );

    // copy static libraries
    log::info!("Copy static libraries");
    let dst = format!("./libs/{chip}/");
    remove_dir_all(&dst);
    mk_dir(&dst);

    // the just built supplicant
    copy_file(
        "./helper_project/build/esp-idf/wpa_supplicant/libwpa_supplicant.a",
        &format!("{dst}/libwpa_supplicant.a"),
    );

    // blobs from ESP-IDF installation
    let idf_path = std::env::var("IDF_PATH").unwrap();

    // phy
    copy_file(
        &format!("{idf_path}/components/esp_phy/lib/{chip}/libphy.a"),
        &format!("{dst}/libphy.a"),
    );

    if chip == "esp32" {
        copy_file(
            &format!("{idf_path}/components/esp_phy/lib/{chip}/librtc.a"),
            &format!("{dst}/librtc.a"),
        );
    }

    if chip != "esp32" && chip != "esp32s2" {
        copy_file(
            &format!("{idf_path}/components/esp_phy/lib/{chip}/libbtbb.a"),
            &format!("{dst}/libbtbb.a"),
        );
    }

    // wifi
    if chip != "esp32h2" {
        copy_file(
            &format!("{idf_path}/components/esp_wifi/lib/{chip}/libcore.a"),
            &format!("{dst}/libcore.a"),
        );
        copy_file(
            &format!("{idf_path}/components/esp_wifi/lib/{chip}/libpp.a"),
            &format!("{dst}/libpp.a"),
        );
        copy_file(
            &format!("{idf_path}/components/esp_wifi/lib/{chip}/libespnow.a"),
            &format!("{dst}/libespnow.a"),
        );
        copy_file(
            &format!("{idf_path}/components/esp_wifi/lib/{chip}/libmesh.a"),
            &format!("{dst}/libmesh.a"),
        );
        copy_file(
            &format!("{idf_path}/components/esp_wifi/lib/{chip}/libnet80211.a"),
            &format!("{dst}/libnet80211.a"),
        );
        copy_file(
            &format!("{idf_path}/components/esp_wifi/lib/{chip}/libsmartconfig.a"),
            &format!("{dst}/libsmartconfig.a"),
        );
        copy_file(
            &format!("{idf_path}/components/esp_wifi/lib/{chip}/libwapi.a"),
            &format!("{dst}/libwapi.a"),
        );
    }

    // coex
    copy_file(
        &format!("{idf_path}/components/esp_coex/lib/{chip}/libcoexist.a"),
        &format!("{dst}/libcoexist.a"),
    );

    // bt
    match chip {
        "esp32" => {
            copy_file(
                &format!("{idf_path}/components/bt/controller/lib_esp32/esp32/libbtdm_app.a"),
                &format!("{dst}/libbtdm_app.a"),
            );
        }
        "esp32c3" => {
            copy_file(
                &format!(
                    "{idf_path}/components/bt/controller/lib_esp32c3_family/esp32c3/libbtdm_app.a"
                ),
                &format!("{dst}/libbtdm_app.a"),
            );
        }
        "esp32s3" => {
            copy_file(
                &format!(
                    "{idf_path}/components/bt/controller/lib_esp32c3_family/esp32s3/libbtdm_app.a"
                ),
                &format!("{dst}/libbtdm_app.a"),
            );
        }
        "esp32s2" => (),
        "esp32c2" => {
            copy_file(
                &format!(
                    "{idf_path}/components/bt/controller/lib_esp32c2/esp32c2-bt-lib/libble_app.a"
                ),
                &format!("{dst}/libble_app.a"),
            );
        }
        "esp32c6" => {
            copy_file(
                &format!(
                    "{idf_path}/components/bt/controller/lib_esp32c6/esp32c6-bt-lib/libble_app.a"
                ),
                &format!("{dst}/libble_app.a"),
            );
        }
        "esp32h2" => {
            copy_file(
                &format!(
                    "{idf_path}/components/bt/controller/lib_esp32h2/esp32h2-bt-lib/libble_app.a"
                ),
                &format!("{dst}/libble_app.a"),
            );
        }

        _ => panic!("Unknown chip to copy bt libs"),
    }

    // copy headers
    log::info!("Copy chip specific headers");
    let dst = format!("./include/{chip}/");
    remove_dir_all(&dst);
    mk_dir(&dst);

    copy_files(
        &format!("{idf_path}/components/esp_phy/{chip}/include"),
        &format!("{dst}"),
    );
    copy_files(
        &format!("{idf_path}/components/bt/include/{chip}/include"),
        &format!("{dst}"),
    );
    copy_file(
        "./helper_project/build/config/sdkconfig.h",
        &format!("{dst}/sdkconfig.h"),
    );

    replace_in_file(&format!("{dst}/esp_bt.h"), r#"#include "esp_task.h""#, r#""#);
    replace_in_file(&format!("{dst}/esp_bt.h"), r#"#include "../../../../controller/esp32c6/esp_bt_cfg.h""#, r#""#);


}

fn remove_dir_all(path: &str) {
    let cwd = env::current_dir().unwrap();
    fs::remove_dir_all(windows_safe_path(&cwd.join(path))).ok();
}

fn remove_file(path: &str) {
    let cwd = env::current_dir().unwrap();
    fs::remove_file(windows_safe_path(&cwd.join(path))).ok();
}

fn copy_file(from: &str, to: &str) {
    let cwd = env::current_dir().unwrap();
    fs::copy(
        windows_safe_path(&cwd.join(from)),
        windows_safe_path(&cwd.join(to)),
    )
    .expect("Unable to copy sdkconfig.defaults");
}

fn copy_files(from: &str, to: &str) {
    let cwd = env::current_dir().unwrap();

    let files: Vec<fs::DirEntry> = windows_safe_path(&cwd.join(from))
        .as_path()
        .read_dir()
        .expect("Unable to read dir")
        .map(|v| v.unwrap())
        .collect();

    let files: Vec<&fs::DirEntry> = files.iter().filter(|v| v.file_type().unwrap().is_file()).collect();

    for file in files {
        let fname = file.file_name().into_string().unwrap();

        fs::copy(
            windows_safe_path(&cwd.join(from).join(&fname)),
            windows_safe_path(&cwd.join(to).join(&fname)),
        )
        .expect("Unable to copy sdkconfig.defaults");
    }
}

fn build(cwd: &str, args: &[&str]) {
    #[cfg(target_os = "windows")]
    let cmd = "idf.py.exe";

    #[cfg(not(target_os = "windows"))]
    let cmd = "idf.py";

    let cwd = windows_safe_path(&env::current_dir().unwrap().join(cwd));
    let output = std::process::Command::new(cmd)
        .args(args)
        .current_dir(cwd)
        .stdout(std::process::Stdio::inherit())
        .stdin(std::process::Stdio::inherit())
        .output()
        .expect("Unable to run command {cmd}");

    if !output.status.success() {
        println!(
            "Failed to run build {}",
            str::from_utf8(&output.stderr).unwrap()
        );
    }
}

fn mk_dir(p: &str) {
    let mut cwd = env::current_dir().unwrap();
    let p = windows_safe_path(&cwd.join(p));
    fs::create_dir_all(p).expect("Unable to create libs directory");
}

fn replace_in_file(p: &str, search: &str, replace: &str) {
    let mut cwd = env::current_dir().unwrap();
    let p = windows_safe_path(&cwd.join(p));

    let original = fs::read_to_string(&p).unwrap();
    let new = &original.replace(search, replace);
    fs::write(&p, new).unwrap();
}

/// Make the path "Windows"-safe
fn windows_safe_path(path: &Path) -> PathBuf {
    PathBuf::from(path.to_str().unwrap().to_string().replace("\\\\?\\", ""))
}
