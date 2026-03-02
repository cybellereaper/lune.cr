$ErrorActionPreference = 'Stop'

function Invoke-Step {
  param(
    [Parameter(Mandatory = $true)]
    [scriptblock]$Command,
    [Parameter(Mandatory = $true)]
    [string]$Description
  )

  & $Command
  if ($LASTEXITCODE -ne 0) {
    throw "$Description failed with exit code $LASTEXITCODE"
  }
}

$BuildDir = if ($env:BUILD_DIR) { $env:BUILD_DIR } else { 'build' }
$BuildType = if ($env:BUILD_TYPE) { $env:BUILD_TYPE } else { 'Release' }
$PackageDir = if ($env:PACKAGE_DIR) { $env:PACKAGE_DIR } else { 'dist' }

$LlvmCandidates = @()
if ($env:LLVM_DIR) {
  $LlvmCandidates += $env:LLVM_DIR
}
$LlvmCandidates += @(
  "$env:ProgramFiles/LLVM/lib/cmake/llvm",
  "$env:ProgramFiles/LLVM/lib/cmake/LLVM"
)

$LlvmDir = $null
foreach ($Candidate in $LlvmCandidates) {
  if ($Candidate -and (Test-Path (Join-Path $Candidate 'LLVMConfig.cmake'))) {
    $LlvmDir = $Candidate
    break
  }
}

if (-not $LlvmDir) {
  throw "Unable to locate LLVMConfig.cmake. Set LLVM_DIR to the directory containing LLVMConfig.cmake. Checked: $($LlvmCandidates -join ', ')"
}

Write-Output "Using LLVM_DIR=$LlvmDir"

Invoke-Step -Description 'CMake configure' -Command {
  cmake -S . -B $BuildDir -DLUNE_BUILD_TESTS=ON -DLLVM_DIR="$LlvmDir"
}

Invoke-Step -Description 'CMake build' -Command {
  cmake --build $BuildDir --config $BuildType
}

Invoke-Step -Description 'CTest execution' -Command {
  ctest --test-dir $BuildDir -C $BuildType --output-on-failure
}

New-Item -ItemType Directory -Path $PackageDir -Force | Out-Null

$ExePath = Join-Path $BuildDir "$BuildType/lune.exe"
if (-not (Test-Path $ExePath)) {
  throw "Expected executable was not found at '$ExePath'."
}

Copy-Item $ExePath "$PackageDir/lune-windows-x64.exe"

Compress-Archive -Path "$PackageDir/lune-windows-x64.exe" -DestinationPath "$PackageDir/lune-windows-x64.zip" -Force
Write-Output "Built artifact: $PackageDir/lune-windows-x64.zip"
