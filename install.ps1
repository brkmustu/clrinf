# ==============================================================================
# clrinf CLI & MCP Windows Kurulum Betiği (PowerShell)
# Windows 10, 11 ve Windows Server sürümleriyle tam uyumludur.
# ==============================================================================

[CmdletBinding(DefaultParameterSetName = "Install")]
param (
    [Parameter(ParameterSetName = "Install")]
    [switch]$User,

    [Parameter(ParameterSetName = "Install")]
    [switch]$System,

    [Parameter(ParameterSetName = "Install")]
    [string]$Dir,

    [Parameter(ParameterSetName = "Install")]
    [switch]$Build,

    [Parameter(ParameterSetName = "Install")]
    [switch]$Uninstall,

    [Parameter(ParameterSetName = "Help")]
    [Alias("h")]
    [switch]$Help
)

$ErrorActionPreference = "Stop"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

if ($Help) {
    Write-Host "clrinf CLI & MCP Windows Installer" -ForegroundColor Cyan
    Write-Host "Compatible with Windows 10, 11, and Windows Server.`n"
    Write-Host "USAGE:" -ForegroundColor Yellow
    Write-Host "    .\install.ps1 [OPTIONS]`n"
    Write-Host "OPTIONS:" -ForegroundColor Yellow
    Write-Host "    -User                Install to user bin (%LOCALAPPDATA%\clrinf\bin) [Default]"
    Write-Host "    -System              Install system-wide (%ProgramFiles%\clrinf\bin) [Requires Administrator]"
    Write-Host "    -Dir <PATH>          Custom installation directory"
    Write-Host "    -Build               Force build from source (cargo build --release)"
    Write-Host "    -Uninstall           Remove installed binaries and clean PATH"
    Write-Host "    -Help, -h            Show this help message`n"
    exit 0
}

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$CodegenDir = Join-Path $ScriptDir "tools\clrinf-codegen"

$DefaultUserBin = Join-Path $env:LOCALAPPDATA "clrinf\bin"
$DefaultSystemBin = Join-Path $env:ProgramFiles "clrinf\bin"

# Kurulum dizinini belirle
if (-not [string]::IsNullOrWhiteSpace($Dir)) {
    $InstallDir = $Dir
} elseif ($System) {
    $InstallDir = $DefaultSystemBin
} else {
    $InstallDir = $DefaultUserBin
}

$GithubRepo = "brkmustu/clrinf"
$ClrinfVersion = if ($env:CLRINF_VERSION) { $env:CLRINF_VERSION } else { "latest" }

function Test-IsAdmin {
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($identity)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Remove-PathEntry {
    param(
        [Parameter(Mandatory=$true)][string]$TargetDirectory,
        [Parameter(Mandatory=$true)][string]$Scope
    )
    try {
        $regPath = [Environment]::GetEnvironmentVariable("Path", $Scope)
        if (-not [string]::IsNullOrWhiteSpace($regPath)) {
            $normalized = $TargetDirectory.TrimEnd('\')
            $parts = $regPath -split ';' | Where-Object {
                $_.TrimEnd('\') -ne $normalized -and -not [string]::IsNullOrWhiteSpace($_)
            }
            $newPath = $parts -join ';'
            [Environment]::SetEnvironmentVariable("Path", $newPath, $Scope)
        }
    } catch {
        Write-Warning "PATH kaydı güncellenirken hata oluştu: $_"
    }
}

function Add-PathEntry {
    param(
        [Parameter(Mandatory=$true)][string]$TargetDirectory,
        [Parameter(Mandatory=$true)][string]$Scope
    )
    try {
        $regPath = [Environment]::GetEnvironmentVariable("Path", $Scope)
        $parts = if (-not [string]::IsNullOrWhiteSpace($regPath)) { $regPath -split ';' } else { @() }
        $normalized = $TargetDirectory.TrimEnd('\')
        $alreadyExists = $false
        foreach ($p in $parts) {
            if ($p.TrimEnd('\') -eq $normalized) {
                $alreadyExists = $true
                break
            }
        }
        if (-not $alreadyExists) {
            $parts += $TargetDirectory
            $newPath = ($parts | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }) -join ';'
            [Environment]::SetEnvironmentVariable("Path", $newPath, $Scope)
            Write-Host "✅ Added '$TargetDirectory' to [$Scope] PATH." -ForegroundColor Green
        }
    } catch {
        Write-Warning "Failed to update PATH environment variable: $_"
    }
}

# ==============================================================================
# UNINSTALL FLOW
# ==============================================================================
if ($Uninstall) {
    Write-Host "🧹 Removing clrinf CLI and tools..." -ForegroundColor Yellow

    if ($System -and -not (Test-IsAdmin)) {
        Write-Host "❌ System-wide uninstall requires running PowerShell as Administrator." -ForegroundColor Red
        exit 1
    }

    $filesToRemove = @(
        (Join-Path $InstallDir "clrinf-codegen.exe"),
        (Join-Path $InstallDir "clrinf.exe")
    )

    foreach ($file in $filesToRemove) {
        if (Test-Path $file) {
            Remove-Item -Path $file -Force -ErrorAction SilentlyContinue
            Write-Host "✓ Removed: $file" -ForegroundColor Gray
        }
    }

    # Check ~/.cargo/bin
    $cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
    if (Test-Path $cargoBin) {
        $cargoFiles = @(
            (Join-Path $cargoBin "clrinf-codegen.exe"),
            (Join-Path $cargoBin "clrinf.exe")
        )
        foreach ($file in $cargoFiles) {
            if (Test-Path $file) {
                Remove-Item -Path $file -Force -ErrorAction SilentlyContinue
                Write-Host "✓ Removed: $file" -ForegroundColor Gray
            }
        }
    }

    # Clean PATH
    $scope = if ($System) { "Machine" } else { "User" }
    Remove-PathEntry -TargetDirectory $InstallDir -Scope $scope

    # Remove empty directory
    if ((Test-Path $InstallDir) -and ((Get-ChildItem -Path $InstallDir -Force | Measure-Object).Count -eq 0)) {
        Remove-Item -Path $InstallDir -Force -Recurse -ErrorAction SilentlyContinue
    }

    Write-Host "✅ clrinf has been uninstalled." -ForegroundColor Green
    exit 0
}

# ==============================================================================
# INSTALL FLOW
# ==============================================================================
Write-Host "🚀 Starting clrinf Windows Installer..." -ForegroundColor Cyan

if ($System -and -not (Test-IsAdmin)) {
    Write-Host "❌ System-wide install (-System) requires running PowerShell as Administrator." -ForegroundColor Red
    exit 1
}

$TargetBin = $null

function Get-PlatformTriple {
    $arch = if ([System.Environment]::Is64BitOperatingSystem) {
        if ($env:PROCESSOR_ARCHITECTURE -match "ARM64") { "aarch64" } else { "x86_64" }
    } else {
        "x86"
    }
    return "${arch}-pc-windows-msvc"
}

# 1. Önceden derlenmiş ikili dosyayı çözümleme
if (-not $Build) {
    # İlk olarak çalışma alanında önceden derlenmiş ikili dosyayı ara
    $possibleBins = @(
        (Join-Path $CodegenDir "target\release\clrinf-codegen.exe"),
        (Join-Path $ScriptDir "target\release\clrinf-codegen.exe")
    )

    foreach ($bin in $possibleBins) {
        if (Test-Path $bin) {
            $TargetBin = $bin
            Write-Host "✓ Found locally built binary: $TargetBin" -ForegroundColor Green
            break
        }
    }

    # If no local binary, try downloading pre-built binary from GitHub Releases
    if (-not $TargetBin) {
        $platform = Get-PlatformTriple
        Write-Host "🌐 Checking GitHub Releases for pre-built binary ($platform)..." -ForegroundColor Blue

        $urls = @(
            "https://github.com/$GithubRepo/releases/$(if ($ClrinfVersion -eq 'latest') { 'latest/download' } else { "download/$ClrinfVersion" })/clrinf-$platform.zip",
            "https://github.com/$GithubRepo/releases/$(if ($ClrinfVersion -eq 'latest') { 'latest/download' } else { "download/$ClrinfVersion" })/clrinf-$platform.tar.gz"
        )

        $tempFolder = Join-Path ([System.IO.Path]::GetTempPath()) ("clrinf_install_" + [System.Guid]::NewGuid().ToString("N"))
        New-Item -ItemType Directory -Path $tempFolder -Force | Out-Null

        foreach ($url in $urls) {
            $isZip = $url.EndsWith(".zip")
            $archiveFile = Join-Path $tempFolder ("clrinf_package" + $(if ($isZip) { ".zip" } else { ".tar.gz" }))

            try {
                Invoke-WebRequest -Uri $url -OutFile $archiveFile -UseBasicParsing -ErrorAction Stop
                if ((Test-Path $archiveFile) -and ((Get-Item $archiveFile).Length -gt 0)) {
                    Write-Host "✓ Downloaded pre-built archive. Extracting..." -ForegroundColor Green
                    if ($isZip) {
                        Expand-Archive -Path $archiveFile -DestinationPath $tempFolder -Force
                    } else {
                        if (Get-Command tar.exe -ErrorAction SilentlyContinue) {
                            tar.exe -xzf $archiveFile -C $tempFolder
                        }
                    }

                    $extractedCandidate = Join-Path $tempFolder "clrinf-codegen.exe"
                    if (-not (Test-Path $extractedCandidate)) {
                        $extractedCandidate = Join-Path $tempFolder "clrinf.exe"
                    }

                    if (Test-Path $extractedCandidate) {
                        $TargetBin = $extractedCandidate
                        break
                    }
                }
            } catch {
                # Fall through to next URL or source build fallback
            }
        }

        if (-not $TargetBin) {
            Write-Host "ℹ️  Pre-built GitHub Release package not found (not published yet or offline)." -ForegroundColor Yellow
        }
    }
}

# Fallback: Build from source if binary is still not found or -Build was passed
if (-not $TargetBin -or $Build) {
    $cargoToml = Join-Path $CodegenDir "Cargo.toml"
    if ((Test-Path $CodegenDir) -and (Test-Path $cargoToml)) {
        Write-Host "📦 Building clrinf-codegen from local source (cargo build --release)..." -ForegroundColor Blue

        if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
            Write-Host "❌ 'cargo' not found! Rust is required to build from source." -ForegroundColor Red
            Write-Host "👉 Install Rust via https://rustup.rs or 'winget install Rustlang.Rustup'." -ForegroundColor Yellow
            exit 1
        }

        Push-Location $CodegenDir
        try {
            cargo build --release
            if ($LASTEXITCODE -ne 0) {
                Write-Host "❌ Build failed." -ForegroundColor Red
                exit $LASTEXITCODE
            }
        } finally {
            Pop-Location
        }

        $builtBin = Join-Path $CodegenDir "target\release\clrinf-codegen.exe"
        if (Test-Path $builtBin) {
            $TargetBin = $builtBin
        }
    } else {
        Write-Host "❌ No pre-built binary found and local source directory is unavailable." -ForegroundColor Red
        Write-Host "👉 Clone the repository and build from source:" -ForegroundColor Yellow
        Write-Host "   git clone https://github.com/$GithubRepo.git"
        Write-Host "   cd clrinf; .\install.ps1 -Build"
        exit 1
    }
}

if (-not $TargetBin -or -not (Test-Path $TargetBin)) {
    Write-Host "❌ Could not verify binary: $TargetBin" -ForegroundColor Red
    exit 1
}

# 2. Prepare destination directory and copy binaries
Write-Host "📁 Preparing install directory: $InstallDir" -ForegroundColor Blue
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

$destCodegen = Join-Path $InstallDir "clrinf-codegen.exe"
$destClrinf = Join-Path $InstallDir "clrinf.exe"

Copy-Item -Path $TargetBin -Destination $destCodegen -Force
Copy-Item -Path $TargetBin -Destination $destClrinf -Force
Write-Host "✓ Installed binaries:" -ForegroundColor Green
Write-Host "    $destCodegen" -ForegroundColor Gray
Write-Host "    $destClrinf" -ForegroundColor Gray

# Mirror into ~/.cargo/bin if available
$cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
if ((Test-Path $cargoBin) -and ($InstallDir.TrimEnd('\') -ne $cargoBin.TrimEnd('\'))) {
    try {
        Copy-Item -Path $TargetBin -Destination (Join-Path $cargoBin "clrinf-codegen.exe") -Force -ErrorAction SilentlyContinue
        Copy-Item -Path $TargetBin -Destination (Join-Path $cargoBin "clrinf.exe") -Force -ErrorAction SilentlyContinue
        Write-Host "✓ Linked into Cargo bin: $cargoBin" -ForegroundColor Gray
    } catch {
        # Ignore if Cargo bin copy fails
    }
}

# 3. Verify and update PATH
$scope = if ($System) { "Machine" } else { "User" }
Add-PathEntry -TargetDirectory $InstallDir -Scope $scope

# Update PATH for current PowerShell session
$envPathParts = $env:Path -split ';'
$normalizedInstallDir = $InstallDir.TrimEnd('\')
$foundInCurrentPath = $false
foreach ($p in $envPathParts) {
    if ($p.TrimEnd('\') -eq $normalizedInstallDir) {
        $foundInCurrentPath = $true
        break
    }
}
if (-not $foundInCurrentPath) {
    $env:Path = "$env:Path;$InstallDir"
}

Write-Host ""
Write-Host "🎉 clrinf CLI & MCP Server installed successfully!" -ForegroundColor Green
Write-Host "  📍 Binary:   $destCodegen"
Write-Host "  🔗 Alias:    $destClrinf"
Write-Host ""
Write-Host "Quick Verification Commands:" -ForegroundColor Cyan
Write-Host "  clrinf --version"
Write-Host "  clrinf verify --help"
Write-Host "  clrinf catalog"
Write-Host "  clrinf mcp --profile lean"
Write-Host ""
