cmake_minimum_required(VERSION 3.31)

project(
  CircomWitnessGenerator
  LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

list(APPEND CMAKE_MODULE_PATH "${PROJECT_SOURCE_DIR}/cmake")

find_package(GMP REQUIRED)
find_package(Threads REQUIRED)

add_executable({{run_name}} {{run_name}}.cpp calcwit.cpp fr.cpp main.cpp)
target_include_directories({{run_name}} PUBLIC ${GMP_C_INCLUDES})
target_link_libraries({{run_name}} PUBLIC ${GMP_C_LIBRARIES} Threads::Threads)