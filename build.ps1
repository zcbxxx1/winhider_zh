# =============================================================================
# WinHider Rust Build Script
# =============================================================================
#
# DESCRIPTION:
#   Automates the build process for the WinHider application on Windows.
#   It ensures the necessary Rust target (x86_64-pc-windows-msvc) is installed,
#   compiles the project, and opens the build directories for easy access to artifacts.
#
# ARGUMENTS:
#   --nodebug    Skipps the debug build configuration. Only builds the Release version.
#                Useful for CI/CD pipelines or final production builds to save time.
#
# WORKFLOW:
#   1. Sets working directory to script location.
#   2. Parses arguments (checks for --nodebug).
#   3. Installs/Updates the x86_64-pc-windows-msvc Rust target.
#   4. Iterates through configurations (Debug/Release):
#      - Runs 'cargo build' with specific target flags.
#      - Tracks success/failure status.
#   5. Prints a color-coded build summary.
#   6. Opens the build directories (debug/release) in Windows Explorer.
#
# =============================================================================

$scriptDir = Split-Path -Path $MyInvocation.MyCommand.Definition -Parent
Set-Location $scriptDir
$ErrorActionPreference = "Stop"

# ---------------------------
# Flags
# ---------------------------
$skipDebug = $false
if ($args -contains "--nodebug") {
    $skipDebug = $true
    Write-Host "`n--nodebug flag detected. Skipping Debug builds..." -ForegroundColor Yellow
}

# ---------------------------
# Ensure x64 target exists
# ---------------------------
Write-Host "Checking Rust x64 target..." -ForegroundColor Cyan
rustup target add x86_64-pc-windows-msvc | Out-Null

# ---------------------------
# Configs
# ---------------------------
$configurations = if ($skipDebug) { @("release") } else { @("debug", "release") }
$target = "x86_64-pc-windows-msvc"

$buildStatus = @{}

# ---------------------------
# Build Loop
# ---------------------------
foreach ($config in $configurations) {

    $key = "x64-$config"
    Write-Host "`nBuilding for x64 ($target) - $config" -ForegroundColor Cyan

    if ($config -eq "release") {
        cargo build --target $target --release
    } else {
        cargo build --target $target
    }

    if ($LASTEXITCODE -eq 0) {
        $buildStatus[$key] = "Success"
    } else {
        $buildStatus[$key] = "Failed (Exit Code: $LASTEXITCODE)"
    }
}

# ---------------------------
# Summary
# ---------------------------
Write-Host "`n=== Build Summary ===" -ForegroundColor Yellow
foreach ($entry in $buildStatus.GetEnumerator()) {
    $color = if ($entry.Value -like "Success*") { "Green" } else { "Red" }
    Write-Host ("{0,-15} : {1}" -f $entry.Key, $entry.Value) -ForegroundColor $color
}

# ---------------------------
# Open build directories
# ---------------------------
Write-Host "`nOpening build directories..." -ForegroundColor Cyan
foreach ($config in $configurations) {
    $buildDir = "target\$target\$config"
    if (Test-Path $buildDir) {
        Write-Host "Opening $config build directory..." -ForegroundColor Green
        explorer.exe $buildDir
    } else {
        Write-Host "Build directory $buildDir not found" -ForegroundColor Red
    }
}

pause
exit 0