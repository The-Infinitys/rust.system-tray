cmake_minimum_required(VERSION 3.16)

project(qt6-bind VERSION 0.1 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

option(ENABLE_LINTER "Enable clang-tidy linter" ON)
option(ENABLE_FORMATTER "Enable clang-format formatter" ON)

set(CMAKE_AUTOMOC ON)

find_package(Qt6 REQUIRED COMPONENTS Core Gui Widgets)
qt_standard_project_setup()

add_library(qt6-bind STATIC
        src/lib.cpp
)
target_include_directories(qt6-bind PUBLIC src)
target_link_libraries(qt6-bind PRIVATE Qt${QT_VERSION_MAJOR}::Widgets)

# ---
## Add an executable for testing

# ---
include(GNUInstallDirs)
install(TARGETS qt6-bind
    LIBRARY DESTINATION ${CMAKE_INSTALL_LIBDIR}
    ARCHIVE DESTINATION ${CMAKE_INSTALL_LIBDIR}
)