# The following lines of boilerplate have to be in your project's
# CMakeLists in this exact order for cmake to work correctly
cmake_minimum_required(VERSION 3.5)

include($ENV{IDF_PATH}/tools/cmake/project.cmake)
project(helper_project)

# CONFIG_APP_REPRODUCIBLE_BUILD adds one -fdebug-prefix-map per component.
# ESP-IDF 6.1 split HAL into many esp_hal_* components, so those flags overflow
# Windows CreateProcess (~32k). The toolchain already uses a response file;
# move the prefix maps into it so ninja's compile lines stay short.
idf_build_get_property(compile_options COMPILE_OPTIONS)
set(_kept_compile_options)
set(_prefix_map_options)
foreach(_opt IN LISTS compile_options)
    if(_opt MATCHES "^-f(debug|macro)-prefix-map=")
        list(APPEND _prefix_map_options "${_opt}")
    else()
        list(APPEND _kept_compile_options "${_opt}")
    endif()
endforeach()
if(_prefix_map_options)
    idf_build_set_property(COMPILE_OPTIONS "${_kept_compile_options}")
    foreach(_rsp cflags cxxflags asmflags)
        set(_rsp_path "${CMAKE_BINARY_DIR}/toolchain/${_rsp}")
        if(EXISTS "${_rsp_path}")
            foreach(_opt IN LISTS _prefix_map_options)
                file(APPEND "${_rsp_path}" "\n${_opt}")
            endforeach()
        endif()
    endforeach()
endif()

# The supplicant's WPA3-Personal (SAE) is built on mbedTLS / PSA crypto
# (CONFIG_ESP_WIFI_MBEDTLS_CRYPTO=y). Whenever the nvs_flash component is in
# the build, the mbedtls component compiles PSA's persistent-key storage
# (port/psa_crypto_storage/esp_psa_its.c over NVS) into libmbedcrypto.a and
# defines ESP_PSA_ITS_AVAILABLE, which keeps MBEDTLS_PSA_CRYPTO_STORAGE_C on.
# The host has neither NVS nor a need for persistent keys (SAE keys are
# volatile), so take the storage back out: with CONFIG_MBEDTLS_FS_IO=n and no
# ESP_PSA_ITS_AVAILABLE, esp_config.h undefines MBEDTLS_PSA_CRYPTO_STORAGE_C.
if(CONFIG_ESP_WIFI_MBEDTLS_CRYPTO AND TARGET tfpsacrypto)
    foreach(_prop COMPILE_DEFINITIONS INTERFACE_COMPILE_DEFINITIONS)
        get_target_property(_defs tfpsacrypto ${_prop})
        if(_defs)
            list(REMOVE_ITEM _defs ESP_PSA_ITS_AVAILABLE)
            set_target_properties(tfpsacrypto PROPERTIES ${_prop} "${_defs}")
        endif()
    endforeach()
    get_target_property(_srcs tfpsacrypto SOURCES)
    if(_srcs)
        list(FILTER _srcs EXCLUDE REGEX "esp_psa_its\\.c$")
        set_target_properties(tfpsacrypto PROPERTIES SOURCES "${_srcs}")
    endif()
endif()