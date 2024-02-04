cmake_minimum_required(VERSION 3.22)

project(verona-rt-sys-bindings)

include(FetchContent)

if(CMAKE_GENERATOR STREQUAL "Ninja")
    set(CMAKE_COLOR_DIAGNOSTICS ON)
endif()

# We override snmalloc to use our submodule, so that we don't
# need to hit the network to build. This matters for docs.rs.
# See #18 for details.
set(SNMALLOC_PATH "../snmalloc/")
FetchContent_Declare(snmalloc SOURCE_DIR "${CMAKE_CURRENT_LIST_DIR}/${SNMALLOC_PATH}")

set(VERONA_PATH "../verona-rt/")
add_subdirectory(${VERONA_PATH} verona-rt)


find_package(Threads REQUIRED)
add_library(boxcar_bindings STATIC bindings.cc)
target_link_libraries(boxcar_bindings verona_rt)
set_property(TARGET boxcar_bindings PROPERTY CXX_STANDARD 17)

# add_executable(playground playground.cc)
# target_link_libraries(playground verona_rt)

install(TARGETS boxcar_bindings DESTINATION .)
