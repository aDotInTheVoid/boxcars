cmake_minimum_required(VERSION 3.22)

project(verona-rt-sys-bindings)

if(CMAKE_GENERATOR STREQUAL "Ninja")
    set(CMAKE_COLOR_DIAGNOSTICS ON)
endif()

# Don't build all the runtimes tests to install.
# set(CMAKE_SKIP_INSTALL_ALL_DEPENDENCY true)

set(VERONA_PATH "../../../verona-rt/")

add_subdirectory(${VERONA_PATH} verona-rt)

find_package(Threads REQUIRED)
add_library(boxcar_bindings STATIC bindings.cc)
target_link_libraries(boxcar_bindings verona_rt)

add_executable(playground playground.cc)
target_link_libraries(playground verona_rt)

install(TARGETS boxcar_bindings DESTINATION .)
