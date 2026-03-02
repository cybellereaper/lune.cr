$ErrorActionPreference = 'Stop'

$BuildDir = if ($env:BUILD_DIR) { $env:BUILD_DIR } else { 'build' }
$BuildType = if ($env:BUILD_TYPE) { $env:BUILD_TYPE } else { 'Release' }
$PackageDir = if ($env:PACKAGE_DIR) { $env:PACKAGE_DIR } else { 'dist' }
$LlvmDir = if ($env:LLVM_DIR) { $env:LLVM_DIR } else { "$env:ProgramFiles/LLVM/lib/cmake/llvm" }

if (-not (Test-Path $LlvmDir)) {
  throw "LLVM CMake config directory was not found at '$LlvmDir'. Set LLVM_DIR to the directory containing LLVMConfig.cmake."
}

cmake -S . -B $BuildDir -DLUNE_BUILD_TESTS=ON -DLLVM_DIR="$LlvmDir"
cmake --build $BuildDir --config $BuildType
if ($LASTEXITCODE -ne 0) { throw 'cmake build failed' }

ctest --test-dir $BuildDir -C $BuildType --output-on-failure
if ($LASTEXITCODE -ne 0) { throw 'ctest failed' }

New-Item -ItemType Directory -Path $PackageDir -Force | Out-Null

$ExeCandidates = @(
  "$BuildDir/$BuildType/lune.exe",
  "$BuildDir/lune.exe"
)
$ExePath = $ExeCandidates | Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $ExePath) {
  throw 'Could not locate lune.exe after build'
}

Copy-Item $ExePath "$PackageDir/lune-windows-x64.exe"
Compress-Archive -Path "$PackageDir/lune-windows-x64.exe" -DestinationPath "$PackageDir/lune-windows-x64.zip" -Force
Write-Output "Built artifact: $PackageDir/lune-windows-x64.zip"
