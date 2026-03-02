#!/usr/bin/env bash
set -euo pipefail

BUILD_DIR="${BUILD_DIR:-build}"
BUILD_TYPE="${BUILD_TYPE:-Release}"
PACKAGE_DIR="${PACKAGE_DIR:-dist}"

cmake -S . -B "${BUILD_DIR}" -DCMAKE_BUILD_TYPE="${BUILD_TYPE}" -DLUNE_BUILD_TESTS=ON
cmake --build "${BUILD_DIR}" --config "${BUILD_TYPE}" -j
ctest --test-dir "${BUILD_DIR}" --output-on-failure

mkdir -p "${PACKAGE_DIR}"
cp "${BUILD_DIR}/lune" "${PACKAGE_DIR}/lune-linux-x64"

ARCHIVE="lune-linux-x64.tar.gz"
tar -C "${PACKAGE_DIR}" -czf "${PACKAGE_DIR}/${ARCHIVE}" "lune-linux-x64"

echo "Built artifact: ${PACKAGE_DIR}/${ARCHIVE}"
