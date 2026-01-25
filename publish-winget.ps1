# =============================================================================
# WinHider Winget Automation Script
# =============================================================================

param(
    [string]$Version,
    [string]$Token
)

# --------------------------- Configuration ---------------------------
$PackageId = "Bitmutex.WinHider"
$RepoUrl   = "https://github.com/aamitn/winhider"
$ManifestBaseDir = ".\manifests\b\Bitmutex\WinHider"

# --------------------------- Helper Functions ---------------------------

function Write-Step {
    param([string]$Message)
    Write-Host "`n[$([DateTime]::Now.ToString('HH:mm:ss'))] $Message" -ForegroundColor Cyan
}

function Write-Success {
    param([string]$Message)
    Write-Host "SUCCESS: $Message" -ForegroundColor Green
}

function Write-ErrorExit {
    param([string]$Message)
    Write-Host "ERROR: $Message" -ForegroundColor Red
    exit 1
}

# --------------------------- Input Handling ---------------------------

# 1. Get Version
if ([string]::IsNullOrWhiteSpace($Version)) {
    $Version = Read-Host "Enter the new version number (e.g., 1.0.8)"
}
if ($Version -notmatch '^\d+\.\d+\.\d+$') {
    Write-ErrorExit "Invalid version format '$Version'. Expected format: x.x.x"
}

# 2. Get GitHub Token (Fixed Logic)
if ([string]::IsNullOrWhiteSpace($Token)) {
    # Check environment variable first
    if ($env:GITHUB_TOKEN) {
        $Token = $env:GITHUB_TOKEN
        Write-Host "Using GitHub Token from environment variable." -ForegroundColor Gray
    } else {
        # FIX: Simplified SecureString conversion using NetworkCredential
        $SecureToken = Read-Host "Enter your GitHub PAT (Personal Access Token)" -AsSecureString
        $Token = [System.Net.NetworkCredential]::new('', $SecureToken).Password
    }
}

if ([string]::IsNullOrWhiteSpace($Token)) {
    Write-ErrorExit "GitHub Token is required to submit a PR."
}

# --------------------------- Execution ---------------------------

Write-Step "Step 1: Updating Manifest for $PackageId v$Version..."

$DownloadUrl = "$RepoUrl/releases/download/v$Version/WinhiderInstaller.exe|x64"
$ManifestDir = "$ManifestBaseDir\$Version"
Write-Host "Target URL: $DownloadUrl" -ForegroundColor Gray

# Define arguments for the update command
$UpdateArgs = @(
    "update", 
    $PackageId, 
    "--version", $Version, 
    "--urls", "`"$DownloadUrl`"",
    "--token", $Token
)

try {
    # Run wingetcreate update
    $proc = Start-Process -FilePath "wingetcreate" -ArgumentList $UpdateArgs -NoNewWindow -PassThru -Wait
    if ($proc.ExitCode -ne 0) {
        Write-ErrorExit "wingetcreate update failed. Please check the URL and your Token permissions."
    }
} catch {
    Write-ErrorExit "Failed to execute wingetcreate. Is it installed or in the current folder?"
}

# --------------------------- Verify Manifest ---------------------------

# Locate the manifest folder (wingetcreate might create it in a different structure)
if (-not (Test-Path $ManifestDir)) {
    # Fallback: Search for the version folder inside 'manifests'
    if (Test-Path "manifests") {
        $FoundDir = Get-ChildItem -Path "manifests" -Recurse -Directory | Where-Object { $_.Name -eq $Version } | Select-Object -First 1
        if ($FoundDir) {
            $ManifestDir = $FoundDir.FullName
        }
    }
}

if (-not (Test-Path $ManifestDir)) {
    Write-ErrorExit "Could not locate the generated manifest folder for version $Version."
}
Write-Success "Manifest generated at: $ManifestDir"

# --------------------------- Submission ---------------------------

Write-Step "Step 2: Submitting Pull Request to winget-pkgs..."

$SubmitArgs = @(
    "submit",
    $ManifestDir,
    "--token", $Token
)

try {
    # Run wingetcreate submit
    $proc = Start-Process -FilePath "wingetcreate" -ArgumentList $SubmitArgs -NoNewWindow -PassThru -Wait
    if ($proc.ExitCode -eq 0) {
        Write-Success "Pull Request submitted successfully!"
        Write-Host "Check your email or GitHub notifications for the PR link." -ForegroundColor Yellow
    } else {
        Write-ErrorExit "Submission failed. Ensure your GitHub Token has 'public_repo' permissions."
    }
} catch {
    Write-ErrorExit "Failed to execute wingetcreate submit command."
}