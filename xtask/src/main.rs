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
        let mut chips: Vec<String> = fs::read_dir("./patch")
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().join("sdkconfig.defaults").is_file())
            .map(|e| e.file_name().into_string().unwrap())
            .collect();
        chips.sort();
        chips
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
        &format!("{idf_path}/components/esp_wifi/include/local"),
        &format!("{dst}/local"),
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
    // ESP-IDF 6.0 removed esp_interface.h; wifi_interface_t is in
    // esp_wifi_types_generic.h (copied with the wifi include tree above).
    copy_file(
        &format!("{idf_path}/components/esp_hw_support/include/esp_private/esp_pmu.h"),
        &format!("{dst}/esp_private/esp_pmu.h"),
    );
    copy_file(
        &format!("{idf_path}/components/esp_hw_support/include/esp_private/esp_modem_clock.h"),
        &format!("{dst}/esp_private/esp_modem_clock.h"),
    );
    copy_file(
        &format!("{idf_path}/components/hal/include/hal/modem_clock_types.h"),
        &format!("{dst}/hal/modem_clock_types.h"),
    );

    copy_file(
        &format!("{idf_path}/components/esp_common/include/esp_bit_defs.h"),
        &format!("{dst}/esp_bit_defs.h"),
    );
    copy_file(
        &format!("{idf_path}/components/esp_common/include/esp_attr.h"),
        &format!("{dst}/esp_attr.h"),
    );
    copy_file(
        &format!("{idf_path}/components/esp_common/include/esp_types.h"),
        &format!("{dst}/esp_types.h"),
    );
    copy_file(
        &format!("{idf_path}/components/esp_common/include/esp_assert.h"),
        &format!("{dst}/esp_assert.h"),
    );
    copy_file(
        &format!("{idf_path}/components/esp_hw_support/etm/include/esp_etm.h"),
        &format!("{dst}/esp_etm.h"),
    );
    copy_file(
        &format!("{idf_path}/components/hal/include/hal/etm_types.h"),
        &format!("{dst}/hal/etm_types.h"),
    );
    copy_file(
        &format!("{idf_path}/components/bt/host/nimble/nimble/nimble/include/nimble/nimble_npl.h"),
        &format!("{dst}/nimble/nimble_npl.h"),
    );
    copy_file(
        &format!(
            "{idf_path}/components/bt/host/nimble/nimble/porting/npl/dummy/include/nimble/nimble_npl_os.h"
        ),
        &format!("{dst}/nimble/nimble_npl_os.h"),
    );

    copy_file(
        &format!("{idf_path}/components/esp_hal_pmu/include/hal/pmu_types.h"),
        &format!("{dst}/hal/pmu_types.h"),
    );
    copy_file(
        &format!("{idf_path}/components/hal/platform_port/include/hal/assert.h"),
        &format!("{dst}/hal/assert.h"),
    );
    copy_file(
        &format!("{idf_path}/components/hal/platform_port/include/hal/misc.h"),
        &format!("{dst}/hal/misc.h"),
    );
    copy_file(
        &format!("{idf_path}/components/hal/include/hal/modem_clock_hal.h"),
        &format!("{dst}/hal/modem_clock_hal.h"),
    );

    replace_in_file(
        &format!("{dst}/hal/modem_clock_hal.h"),
        r#"#include "hal/regi2c_ctrl_ll.h""#,
        "",
    );
    replace_in_file(
        &format!("{dst}/esp_private/esp_modem_clock.h"),
        r#"#include "hal/regi2c_ctrl_ll.h""#,
        "",
    );

    replace_in_file(&format!("{dst}/esp_coexist_internal.h"), "private/", "");
    replace_in_file(
        &format!("{dst}/esp_event.h"),
        r#"#include "freertos/FreeRTOS.h""#,
        r#""#,
    );
    replace_in_file(
        &format!("{dst}/esp_event.h"),
        r#"#include "freertos/task.h""#,
        r#""#,
    );
    replace_in_file(
        &format!("{dst}/esp_event.h"),
        r#"#include "freertos/queue.h""#,
        r#""#,
    );
    replace_in_file(
        &format!("{dst}/esp_event.h"),
        r#"#include "freertos/semphr.h""#,
        r#""#,
    );
    replace_in_file(
        &format!("{dst}/esp_system.h"),
        r#"#include "esp_idf_version.h""#,
        r#""#,
    );
    replace_in_file(
        &format!("{dst}/esp_private/esp_wifi_private.h"),
        r#"#include "freertos/FreeRTOS.h""#,
        r#""#,
    );
    replace_in_file(
        &format!("{dst}/esp_private/esp_wifi_private.h"),
        r#"#include "freertos/queue.h""#,
        r#""#,
    );
    replace_in_file(
        &format!("{dst}/esp_private/wifi.h"),
        r#"#include "freertos/FreeRTOS.h""#,
        r#""#,
    );
    replace_in_file(
        &format!("{dst}/esp_private/wifi.h"),
        r#"#include "freertos/queue.h""#,
        r#""#,
    );
    replace_in_file(
        &format!("{dst}/esp_task.h"),
        r#"#include "freertos/FreeRTOS.h""#,
        r#""#,
    );
    replace_in_file(
        &format!("{dst}/esp_task.h"),
        r#"#include "freertos/FreeRTOSConfig.h""#,
        r#""#,
    );

    // update version file
    let version = idf_version();
    log::info!("ESP-IDF version {}", &version);

    fs::write("version", &version).unwrap();
}

fn process(chip: &str) {
    log::info!("Processing {chip}");

    // clean
    log::info!("Clean");
    remove_dir_all("./helper_project/build");
    remove_file("./helper_project/sdkconfig");
    remove_file("./helper_project/sdkconfig.defaults");
    remove_file("./helper_project/sdkconfig.old");

    // build
    log::info!("Build");

    let idf_path = std::env::var("IDF_PATH").unwrap();
    let patch = format!("./patch/{chip}/sdkconfig.defaults");
    if !Path::new(&patch).exists() {
        let has_wifi = Path::new(&format!("{idf_path}/components/esp_wifi/lib/{chip}")).exists();
        let template = ["esp32c6", "esp32c5", "esp32h2", "esp32h4"]
            .into_iter()
            .find(|c| {
                Path::new(&format!("./patch/{c}/sdkconfig.defaults")).exists()
                    && Path::new(&format!("{idf_path}/components/esp_wifi/lib/{c}")).exists()
                        == has_wifi
            })
            .unwrap_or_else(|| panic!("No sdkconfig.defaults template for {chip}"));
        log::warn!("{chip}: no {patch}, copied it from {template}");
        copy_file(&format!("./patch/{template}/sdkconfig.defaults"), &patch);
    }

    copy_file(&patch, "./helper_project/sdkconfig.defaults");

    let target_arg = format!("-DIDF_TARGET={chip}");
    build("helper_project", &["--preview", &target_arg, "build"]);

    let sdkconfig = fs::read_to_string("./helper_project/build/config/sdkconfig.h").unwrap();
    let has_cfg = |name: &str| sdkconfig.contains(&format!("#define {name} 1"));
    let has_wifi = has_cfg("CONFIG_SOC_WIFI_SUPPORTED");
    let has_bt = has_cfg("CONFIG_SOC_BT_SUPPORTED");
    let is_xtensa = has_cfg("CONFIG_IDF_TARGET_ARCH_XTENSA");

    let dst = format!("./libs/{chip}/");
    remove_dir_all(&dst);
    mk_dir(&dst);

    if has_wifi {
        log::info!("Create libregulatory.a");
        ar(
            is_xtensa,
            "helper_project",
            &[
                &format!("../{dst}/libregulatory.a"),
                "./build/esp-idf/esp_wifi/CMakeFiles/__idf_esp_wifi.dir/regulatory/esp_wifi_regulatory.c.obj",
            ],
        );
    }

    // copy static libraries
    log::info!("Copy static libraries");

    // the printf compat library
    copy_file(
        "./helper_project/build/esp-idf/main/libprintf.a",
        &format!("{dst}/libprintf.a"),
    );

    // the just built supplicant
    if has_wifi {
        copy_file(
            "./helper_project/build/esp-idf/wpa_supplicant/libwpa_supplicant.a",
            &format!("{dst}/libwpa_supplicant.a"),
        );
    }

    // blobs from ESP-IDF installation
    // phy
    for lib in ["libphy.a", "librtc.a", "libbtbb.a"] {
        let src = format!("{idf_path}/components/esp_phy/lib/{chip}/{lib}");
        if Path::new(&src).exists() {
            copy_file(&src, &format!("{dst}/{lib}"));
        }
    }

    // wifi
    if has_wifi {
        copy_files(&format!("{idf_path}/components/esp_wifi/lib/{chip}"), &dst);
    }

    // coex
    copy_file(
        &format!("{idf_path}/components/esp_coex/lib/{chip}/libcoexist.a"),
        &format!("{dst}/libcoexist.a"),
    );

    // bt
    let bt_target = has_bt.then(|| bt_target(&idf_path, chip));
    if let Some(bt_target) = &bt_target {
        let bt_lib_dir = [
            format!(
                "{idf_path}/components/bt/controller/lib_{bt_target}/{bt_target}-bt-lib/{chip}"
            ),
            format!("{idf_path}/components/bt/controller/lib_{bt_target}/{bt_target}-bt-lib"),
            format!("{idf_path}/components/bt/controller/lib_{bt_target}/{chip}"),
            format!("{idf_path}/components/bt/controller/lib_{bt_target}_family/{chip}"),
        ]
        .into_iter()
        .find(|dir| Path::new(dir).is_dir())
        .unwrap_or_else(|| panic!("Unknown chip to copy bt libs: {chip}"));

        for entry in fs::read_dir(&bt_lib_dir).unwrap() {
            let name = entry.unwrap().file_name().into_string().unwrap();
            if name.starts_with("lib") && name.ends_with(".a") && !name.ends_with("_flash.a") {
                copy_file(&format!("{bt_lib_dir}/{name}"), &format!("{dst}/{name}"));
            }
        }
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
    if let Some(bt_target) = &bt_target {
        copy_files(
            &format!("{idf_path}/components/bt/include/{bt_target}/include"),
            &format!("{dst}"),
        );
    }
    copy_file(
        "./helper_project/build/config/sdkconfig.h",
        &format!("{dst}/sdkconfig.h"),
    );
    copy_file(
        &format!("{idf_path}/components/soc/{chip}/include/soc/soc_caps.h"),
        &format!("{dst}/soc/soc_caps.h"),
    );
    copy_file(
        &format!("{idf_path}/components/soc/{chip}/include/soc/periph_defs.h"),
        &format!("{dst}/soc/periph_defs.h"),
    );
    copy_file(
        &format!("{idf_path}/components/soc/{chip}/include/soc/interrupts.h"),
        &format!("{dst}/soc/interrupts.h"),
    );
    copy_file(
        &format!("{idf_path}/components/soc/{chip}/include/soc/clk_tree_defs.h"),
        &format!("{dst}/soc/clk_tree_defs.h"),
    );
    if has_bt {
        replace_in_file(
            &format!("{dst}/esp_bt.h"),
            r#"#include "esp_task.h""#,
            r#""#,
        );
        replace_in_file(
            &format!("{dst}/esp_bt.h"),
            r#"#include "../../../../controller/"#,
            r#"//#include "../../../../controller/"#,
        );
        replace_in_file(
            &format!("{dst}/esp_bt.h"),
            r#"#include "../../../controller/"#,
            r#"//#include "../../../controller/"#,
        );

        let esp_bt = fs::read_to_string(format!("{dst}/esp_bt.h")).unwrap();
        for (sub, header) in [
            ("btdm_common", "btdm_user_cfg.h"),
            ("ble", "ble_user_cfg.h"),
            ("bredr", "bredr_user_cfg.h"),
        ] {
            if esp_bt.contains(header) {
                copy_file(
                    &format!(
                        "{idf_path}/components/bt/porting_btdm/controller/{sub}/include/{header}"
                    ),
                    &format!("{dst}/{header}"),
                );
            }
        }
    }

    copy_file(
        &format!("{idf_path}/components/soc/{chip}/include/soc/soc.h"),
        &format!("{dst}/soc/soc.h"),
    );
    let reg_base_dir = if Path::new(&format!(
        "{idf_path}/components/soc/{chip}/register/soc/reg_base.h"
    ))
    .exists()
    {
        "register"
    } else {
        "include"
    };
    copy_file(
        &format!("{idf_path}/components/soc/{chip}/{reg_base_dir}/soc/reg_base.h"),
        &format!("{dst}/soc/reg_base.h"),
    );

    let soc_pmu_supported = has_cfg("CONFIG_SOC_PMU_SUPPORTED");
    if soc_pmu_supported {
        copy_file(
            &format!("{idf_path}/components/esp_hal_pmu/{chip}/include/hal/pmu_hal.h"),
            &format!("{dst}/hal/pmu_hal.h"),
        );
        copy_file(
            &format!("{idf_path}/components/esp_hal_pmu/{chip}/include/hal/pmu_ll.h"),
            &format!("{dst}/hal/pmu_ll.h"),
        );
        copy_file(
            &format!("{idf_path}/components/soc/{chip}/register/soc/pmu_struct.h"),
            &format!("{dst}/soc/pmu_struct.h"),
        );
        copy_file(
            &format!(
                "{idf_path}/components/esp_hw_support/port/{chip}/private_include/pmu_param.h"
            ),
            &format!("{dst}/pmu_param.h"),
        );
        copy_file(
            &format!(
                "{idf_path}/components/esp_hw_support/port/{chip}/private_include/pmu_bit_defs.h"
            ),
            &format!("{dst}/pmu_bit_defs.h"),
        );
        copy_file(
            &format!("{idf_path}/components/soc/{chip}/register/soc/pmu_reg.h"),
            &format!("{dst}/soc/pmu_reg.h"),
        );
    }

    let modem_lock_is_independent = has_cfg("CONFIG_SOC_MODEM_CLOCK_IS_INDEPENDENT");
    if modem_lock_is_independent {
        copy_file(
            &format!("{idf_path}/components/soc/{chip}/include/modem/modem_syscon_struct.h"),
            &format!("{dst}/modem/modem_syscon_struct.h"),
        );
        copy_file(
            &format!("{idf_path}/components/soc/{chip}/include/modem/modem_lpcon_struct.h"),
            &format!("{dst}/modem/modem_lpcon_struct.h"),
        );
        copy_file(
            &format!("{idf_path}/components/hal/{chip}/include/hal/modem_lpcon_ll.h"),
            &format!("{dst}/hal/modem_lpcon_ll.h"),
        );
        copy_file(
            &format!("{idf_path}/components/hal/{chip}/include/hal/modem_syscon_ll.h"),
            &format!("{dst}/hal/modem_syscon_ll.h"),
        );
    }
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
    std::fs::create_dir_all(windows_safe_path(&cwd.join(to)).parent().unwrap()).unwrap();
    fs::copy(
        windows_safe_path(&cwd.join(from)),
        windows_safe_path(&cwd.join(to)),
    )
    .expect(&format!("Unable to copy {from} to {to}"));
    set_file_mode(&windows_safe_path(&cwd.join(to)));
}

fn copy_files(from: &str, to: &str) {
    let cwd = env::current_dir().unwrap();

    log::debug!("Copy from path {:?}", windows_safe_path(&cwd.join(from)));
    let files: Vec<fs::DirEntry> = windows_safe_path(&cwd.join(from))
        .as_path()
        .read_dir()
        .expect("Unable to read dir")
        .map(|v| v.unwrap())
        .collect();

    let files: Vec<&fs::DirEntry> = files
        .iter()
        .filter(|v| v.file_type().unwrap().is_file())
        .collect();

    for file in files {
        let fname = file.file_name().into_string().unwrap();

        fs::copy(
            windows_safe_path(&cwd.join(from).join(&fname)),
            windows_safe_path(&cwd.join(to).join(&fname)),
        )
        .unwrap_or_else(|e| panic!("Unable to copy {from}/{fname} to {to}: {e}"));
        set_file_mode(&windows_safe_path(&cwd.join(to).join(&fname)));
    }
}

/// Some ESP-IDF blobs carry the executable bit, which `fs::copy` would carry over.
#[cfg(unix)]
fn set_file_mode(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o644)).unwrap();
}

#[cfg(not(unix))]
fn set_file_mode(_path: &Path) {}

fn build(cwd: &str, args: &[&str]) {
    let mut adapted_args = Vec::new();
    #[cfg(target_os = "windows")]
    adapted_args.push("/c");
    #[cfg(target_os = "windows")]
    adapted_args.push("idf.py");
    adapted_args.extend_from_slice(args);
    let args = adapted_args;

    #[cfg(target_os = "windows")]
    let cmd = "cmd";

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
        eprintln!(
            "Failed to run build {}",
            str::from_utf8(&output.stderr).unwrap()
        );
        std::process::exit(output.status.code().unwrap_or(1));
    }
}

fn bt_target(idf_path: &str, chip: &str) -> String {
    let cmake = fs::read_to_string(format!("{idf_path}/components/bt/CMakeLists.txt")).unwrap();
    let symbol = format!("CONFIG_IDF_TARGET_{}", chip.to_ascii_uppercase());
    let lines: Vec<&str> = cmake.lines().map(str::trim).collect();
    lines
        .windows(2)
        .find_map(|pair| {
            let target = pair[1]
                .strip_prefix("set(TARGET_SRC_NAME \"")?
                .split_once('"')?
                .0;
            pair[0]
                .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                .any(|word| word == symbol)
                .then(|| target.to_string())
        })
        .unwrap_or_else(|| chip.to_string())
}

fn ar(is_xtensa: bool, cwd: &str, args: &[&str]) {
    let ar = if is_xtensa {
        "xtensa-esp-elf-ar"
    } else {
        "riscv32-esp-elf-ar"
    };

    let mut args: Vec<&str> = Vec::from(args);
    args.insert(0, "-rc");

    let cwd = windows_safe_path(&env::current_dir().unwrap().join(cwd));
    let output = std::process::Command::new(ar)
        .args(args)
        .current_dir(cwd)
        .stdout(std::process::Stdio::inherit())
        .stdin(std::process::Stdio::inherit())
        .output()
        .expect("Unable to run command {cmd}");

    if !output.status.success() {
        println!(
            "Failed to run {} {}",
            ar,
            str::from_utf8(&output.stderr).unwrap()
        );
    }
}

fn idf_version() -> String {
    #[cfg(target_os = "windows")]
    let cmd = "cmd";
    #[cfg(target_os = "windows")]
    let args = vec!["/c", "idf.py", "--version"];

    #[cfg(not(target_os = "windows"))]
    let cmd = "idf.py";
    #[cfg(not(target_os = "windows"))]
    let args = vec!["--version"];

    let output = std::process::Command::new(cmd)
        .args(args)
        .output()
        .expect("Unable to run command {cmd}");

    if !output.status.success() {
        println!(
            "Failed to run esp-idf {}",
            str::from_utf8(&output.stderr).unwrap()
        );
    }

    let output = str::from_utf8(&output.stdout).unwrap();

    output.to_string()
}

fn mk_dir(p: &str) {
    let cwd = env::current_dir().unwrap();
    let p = windows_safe_path(&cwd.join(p));
    fs::create_dir_all(p).expect("Unable to create libs directory");
}

fn replace_in_file(p: &str, search: &str, replace: &str) {
    let cwd = env::current_dir().unwrap();
    let p = windows_safe_path(&cwd.join(p));

    let original = fs::read_to_string(&p).unwrap();
    let new = &original.replace(search, replace);
    fs::write(&p, new).unwrap();
}

/// Make the path "Windows"-safe
fn windows_safe_path(path: &Path) -> PathBuf {
    PathBuf::from(path.to_str().unwrap().to_string().replace("\\\\?\\", ""))
}
