cmake_minimum_required(VERSION 3.20)
project(lune LANGUAGES C CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)


if(NOT CMAKE_BUILD_TYPE)
  set(CMAKE_BUILD_TYPE Release CACHE STRING "Build type" FORCE)
endif()

option(LUNE_BUILD_TESTS "Build tests" ON)

find_package(LLVM REQUIRED CONFIG)

message(STATUS "Found LLVM ${LLVM_PACKAGE_VERSION}")

add_library(lune_core
    src/lexer.cpp
    src/parser.cpp
    src/codegen.cpp
    src/gc.cpp
)

target_include_directories(lune_core PUBLIC include)
target_include_directories(lune_core SYSTEM PUBLIC ${LLVM_INCLUDE_DIRS})
target_compile_definitions(lune_core PUBLIC ${LLVM_DEFINITIONS})

target_compile_options(lune_core PRIVATE
    $<$<CONFIG:Release>:-O3 -DNDEBUG -march=native>
    $<$<CONFIG:RelWithDebInfo>:-O3 -DNDEBUG -march=native>
)

target_link_libraries(lune_core
    PUBLIC
    LLVMCore
    LLVMOrcJIT
    LLVMExecutionEngine
    LLVMSupport
    LLVMX86CodeGen
    LLVMX86AsmParser
    LLVMX86Desc
    LLVMX86Info
    LLVMTarget
    LLVMTargetParser
    LLVMObject
)

add_executable(lune src/main.cpp)
target_link_libraries(lune PRIVATE lune_core)

if(LUNE_BUILD_TESTS)
  enable_testing()
  add_executable(lune_tests tests/lune_tests.cpp)
  target_link_libraries(lune_tests PRIVATE lune_core)
  add_test(NAME lune_tests COMMAND lune_tests)
endif()
