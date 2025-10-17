cmake_minimum_required(VERSION 3.10)
project(WrapperProject)

# Set C++ standard
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

#### External libraries ####
# Reuse libraries already fetched by ramulator2
set(RAMULATOR_EXT_DIR ${CMAKE_CURRENT_SOURCE_DIR}/../../3rd-party/ramulator2/ext)

message("Using yaml-cpp from ${RAMULATOR_EXT_DIR}/yaml-cpp...")
include_directories(${RAMULATOR_EXT_DIR}/yaml-cpp/include)

message("Using spdlog from ${RAMULATOR_EXT_DIR}/spdlog...")
include_directories(${RAMULATOR_EXT_DIR}/spdlog/include)

message("Using argparse from ${RAMULATOR_EXT_DIR}/argparse...")
include_directories(${RAMULATOR_EXT_DIR}/argparse/include)
##################################

# Set output directories
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin)

# Include directory
include_directories(${CMAKE_CURRENT_SOURCE_DIR}/../../3rd-party/ramulator2/src)

# Find libramulator (platform-independent)
find_library(RAMULATOR_LIBRARY
  NAMES ramulator
  PATHS ${CMAKE_CURRENT_SOURCE_DIR}/../../3rd-party/ramulator2
  NO_DEFAULT_PATH
)

# Add wrapper shared library
add_library(wrapper SHARED CRamualator2Wrapper.cpp)

# Link libramulator using the found library
target_link_libraries(wrapper ${RAMULATOR_LIBRARY})

# Add main executable
add_executable(main main.cpp)

# Add test executable
add_executable(test test.cpp)

# Link against the wrapper and ramulator libraries
target_link_libraries(main wrapper ${RAMULATOR_LIBRARY})
target_link_libraries(test wrapper ${RAMULATOR_LIBRARY})


