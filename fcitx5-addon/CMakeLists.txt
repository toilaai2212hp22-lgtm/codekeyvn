cmake_minimum_required(VERSION 3.16)
project(fcitx5-codekey VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

find_package(Gettext REQUIRED)
find_package(Fcitx5Core 5.0.0 REQUIRED)
# Optional utils cmake helpers
if(DEFINED FCITX_INSTALL_CMAKECONFIG_DIR)
  include("${FCITX_INSTALL_CMAKECONFIG_DIR}/Fcitx5Utils/Fcitx5CompilerSettings.cmake" OPTIONAL)
endif()

# Built by: cargo build --release -p codekey-ffi
set(CODEKEY_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/.." CACHE PATH "CodeKey repo root")
set(CODEKEY_LIB_DIR "${CODEKEY_ROOT}/target/release" CACHE PATH "dir with libcodekey.so")
set(CODEKEY_INCLUDE_DIR "${CODEKEY_ROOT}/include" CACHE PATH "codekey.h")

find_library(CODEKEY_SHARED
  NAMES codekey
  PATHS "${CODEKEY_LIB_DIR}"
  NO_DEFAULT_PATH
  REQUIRED
)
message(STATUS "libcodekey: ${CODEKEY_SHARED}")
message(STATUS "codekey.h : ${CODEKEY_INCLUDE_DIR}")

# Module name = "codekey" → fcitx loads lib as "codekey.so" (no lib prefix)
add_library(codekeymodule MODULE src/codekey.cpp)
set_target_properties(codekeymodule PROPERTIES
  PREFIX ""
  OUTPUT_NAME "codekey"
  LIBRARY_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}"
)

target_include_directories(codekeymodule PRIVATE
  "${CODEKEY_INCLUDE_DIR}"
  "${CMAKE_CURRENT_SOURCE_DIR}/src"
)
target_link_libraries(codekeymodule PRIVATE
  Fcitx5::Core
  "${CODEKEY_SHARED}"
)
# RPATH so the module finds libcodekey next to system lib after install
set_target_properties(codekeymodule PROPERTIES
  INSTALL_RPATH "\$ORIGIN;\$ORIGIN/.."
)

include(GNUInstallDirs)

# Fcitx5 install paths (provided by Fcitx5CoreConfig)
if(NOT FCITX_INSTALL_LIBDIR)
  set(FCITX_INSTALL_LIBDIR "${CMAKE_INSTALL_LIBDIR}")
endif()
if(NOT FCITX_INSTALL_PKGDATADIR)
  set(FCITX_INSTALL_PKGDATADIR "${CMAKE_INSTALL_DATADIR}/fcitx5")
endif()

install(TARGETS codekeymodule
  LIBRARY DESTINATION "${FCITX_INSTALL_LIBDIR}/fcitx5"
)
# Engine shared library (Rust)
install(FILES "${CODEKEY_SHARED}"
  DESTINATION "${CMAKE_INSTALL_LIBDIR}"
  RENAME "libcodekey.so"
)
# Addon registration
install(FILES data/codekey-addon.conf
  DESTINATION "${FCITX_INSTALL_PKGDATADIR}/addon"
  RENAME codekey.conf
)
# Input method entries (Telex + VNI)
install(FILES
  data/codekey.conf
  data/codekey-vni.conf
  DESTINATION "${FCITX_INSTALL_PKGDATADIR}/inputmethod"
)

# Optional: print install summary
install(CODE "
  message(STATUS \"CodeKey Fcitx5 installed.\")
  message(STATUS \"  module : ${FCITX_INSTALL_LIBDIR}/fcitx5/codekey.so\")
  message(STATUS \"  restart: fcitx5 -r   then add CodeKey in fcitx5-configtool\")
")
