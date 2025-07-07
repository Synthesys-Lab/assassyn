cmake_minimum_required(VERSION 3.10)
project(WrapperProject)

# Set C++ standard
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

#### External libraries ####
include(FetchContent)
set(FETCHCONTENT_UPDATES_DISCONNECTED ON CACHE BOOL "Skip updating the external dependencies after populating them for the first time")

message("Configuring yaml-cpp...")
option(YAML_CPP_BUILD_CONTRIB "Enable yaml-cpp contrib in library" OFF)
option(YAML_CPP_BUILD_TOOLS "Enable parse tools" OFF)
option(YAML_BUILD_SHARED_LIBS "Build yaml-cpp as a shared library" OFF)
FetchContent_Declare(
  yaml-cpp                             
  GIT_REPOSITORY https://github.com/jbeder/yaml-cpp.git
  GIT_TAG        yaml-cpp-0.7.0                        
  SOURCE_DIR     ${CMAKE_SOURCE_DIR}/ext/yaml-cpp
)
FetchContent_MakeAvailable(yaml-cpp)
include_directories(${yaml-cpp_SOURCE_DIR}/include)
message("Done configuring yaml-cpp.")

message("Configuring spdlog...")
FetchContent_Declare(
  spdlog                             
  GIT_REPOSITORY https://github.com/gabime/spdlog.git
  GIT_TAG        v1.11.0     
  SOURCE_DIR     ${CMAKE_SOURCE_DIR}/ext/spdlog
)
FetchContent_MakeAvailable(spdlog)
include_directories(${spdlog_SOURCE_DIR}/include)
message("Done configuring spdlog.")

message("Configuring argparse...")
FetchContent_Declare(
    argparse
    GIT_REPOSITORY https://github.com/p-ranav/argparse.git
    GIT_TAG        v2.9     
    SOURCE_DIR     ${CMAKE_SOURCE_DIR}/ext/argparse
)
FetchContent_MakeAvailable(argparse)
include_directories(${argparse_SOURCE_DIR}/include)
message("Done configuring argparse.")
##################################

# Set output directories
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin)

# Include directory
include_directories(${PROJECT_SOURCE_DIR}/include)

# Add wrapper shared library
add_library(wrapper SHARED MyWrapper.cpp)

# Link libramulator (assumes it's in the root directory)
set(RAMULATOR_PATH ${CMAKE_CURRENT_SOURCE_DIR}/../../3rd-party/ramulator2)
target_link_libraries(wrapper ${RAMULATOR_PATH}/libramulator.so)

# Add main executable
add_executable(main main.cpp)

# Link against the wrapper and ramulator libraries
target_link_libraries(main wrapper ${RAMULATOR_PATH}/libramulator.so)