#!/usr/bin/env bash
set -euo pipefail

BUILD_DIR="${BUILD_DIR:-build}"
BUILD_TYPE="${BUILD_TYPE:-Release}"
PACKAGE_DIR="${PACKAGE_DIR:-dist}"

if [[ -z "${LLVM_DIR:-}" ]]; then
  if command -v llvm-config >/dev/null 2>&1; then
    LLVM_DIR="$(llvm-config --cmakedir)"
  elif command -v llvm-config-20 >/dev/null 2>&1; then
    LLVM_DIR="$(llvm-config-20 --cmakedir)"
  elif command -v llvm-config-19 >/dev/null 2>&1; then
    LLVM_DIR="$(llvm-config-19 --cmakedir)"
  elif command -v llvm-config-18 >/dev/null 2>&1; then
    LLVM_DIR="$(llvm-config-18 --cmakedir)"
  fi
fi

CMAKE_ARGS=(
  -S .
  -B "${BUILD_DIR}"
  -DCMAKE_BUILD_TYPE="${BUILD_TYPE}"
  -DLUNE_BUILD_TESTS=ON
)

if [[ -n "${LLVM_DIR:-}" ]]; then
  CMAKE_ARGS+=("-DLLVM_DIR=${LLVM_DIR}")
fi

cmake "${CMAKE_ARGS[@]}"
cmake --build "${BUILD_DIR}" --config "${BUILD_TYPE}" -j
ctest --test-dir "${BUILD_DIR}" --output-on-failure

mkdir -p "${PACKAGE_DIR}"
cp "${BUILD_DIR}/lune" "${PACKAGE_DIR}/lune-linux-x64"

tar -C "${PACKAGE_DIR}" -czf "${PACKAGE_DIR}/lune-linux-x64.tar.gz" "lune-linux-x64"

echo "Built artifact: ${PACKAGE_DIR}/lune-linux-x64.tar.gz"
