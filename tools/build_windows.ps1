$ErrorActionPreference = 'Stop'

$BuildDir = if ($env:BUILD_DIR) { $env:BUILD_DIR } else { 'build' }
$BuildType = if ($env:BUILD_TYPE) { $env:BUILD_TYPE } else { 'Release' }
$PackageDir = if ($env:PACKAGE_DIR) { $env:PACKAGE_DIR } else { 'dist' }

cmake -S . -B $BuildDir -DLUNE_BUILD_TESTS=ON
cmake --build $BuildDir --config $BuildType
ctest --test-dir $BuildDir -C $BuildType --output-on-failure

New-Item -ItemType Directory -Path $PackageDir -Force | Out-Null
Copy-Item "$BuildDir/$BuildType/lune.exe" "$PackageDir/lune-windows-x64.exe"

Compress-Archive -Path "$PackageDir/lune-windows-x64.exe" -DestinationPath "$PackageDir/lune-windows-x64.zip" -Force
Write-Output "Built artifact: $PackageDir/lune-windows-x64.zip"
