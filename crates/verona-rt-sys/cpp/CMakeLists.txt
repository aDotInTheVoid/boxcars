cmake_minimum_required(VERSION 3.22)

project(verona-rt-sys-bindings)

if(CMAKE_GENERATOR STREQUAL "Ninja")
    set(CMAKE_COLOR_DIAGNOSTICS ON)
endif()

set(VERONA_PATH "../../../verona-rt/")

add_subdirectory(${VERONA_PATH} verona-rt)

find_package(Threads REQUIRED)
add_library(bindings STATIC bindings.cc)
target_link_libraries(bindings verona_rt)

add_executable(playground playground.cc)
target_link_libraries(playground verona_rt)

install(TARGETS bindings DESTINATION .)
