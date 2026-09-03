idf_component_register(SRCS "helper_project_main.c"
                    INCLUDE_DIRS "")

# libprintf.a is a compat archive for other platforms. Do not link it into the
# helper ELF: _putchar and __esp_radio_printf are provided by the consumer.
# Component directories are EXCLUDE_FROM_ALL, so depend on the archive so
# `idf.py build` still produces it.
add_library(printf STATIC printf/printf.c)
add_dependencies(${COMPONENT_LIB} printf)
