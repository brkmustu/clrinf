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
    Write-Host "clrinf CLI & MCP Windows Kurulum Aracı" -ForegroundColor Cyan
    Write-Host "Windows 10, 11 ve tüm modern Windows ortamlarıyla tam uyumludur.`n"
    Write-Host "KULLANIM:" -ForegroundColor Yellow
    Write-Host "    .\install.ps1 [SEÇENEKLER]`n"
    Write-Host "SEÇENEKLER:" -ForegroundColor Yellow
    Write-Host "    -User                Kullanıcı dizinine kur (%LOCALAPPDATA%\clrinf\bin) [Varsayılan]"
    Write-Host "    -System              Sistem geneline kur (%ProgramFiles%\clrinf\bin) [Yönetici yetkisi gerektirir]"
    Write-Host "    -Dir <PATH>          Özel bir hedef kurulum dizini belirle"
    Write-Host "    -Build               Kaynak koddan yeniden derlemeyi zorunlu kıl (cargo build --release)"
    Write-Host "    -Uninstall           Yüklü ikili dosyaları kaldır ve PATH ayarlarını temizle"
    Write-Host "    -Help, -h            Bu yardım mesajını göster`n"
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
            Write-Host "✅ '$TargetDirectory' dizini [$Scope] PATH ortam değişkenine eklendi." -ForegroundColor Green
        }
    } catch {
        Write-Warning "PATH ortam değişkeni güncellenemedi: $_"
    }
}

# ==============================================================================
# UNINSTALL AKIŞI
# ==============================================================================
if ($Uninstall) {
    Write-Host "🧹 clrinf CLI ve araçları kaldırılıyor..." -ForegroundColor Yellow

    if ($System -and -not (Test-IsAdmin)) {
        Write-Host "❌ Sistem geneli kaldırma işlemi için PowerShell Yönetici (Administrator) olarak çalıştırılmalıdır." -ForegroundColor Red
        exit 1
    }

    $filesToRemove = @(
        (Join-Path $InstallDir "clrinf-codegen.exe"),
        (Join-Path $InstallDir "clrinf.exe")
    )

    foreach ($file in $filesToRemove) {
        if (Test-Path $file) {
            Remove-Item -Path $file -Force -ErrorAction SilentlyContinue
            Write-Host "✓ Silindi: $file" -ForegroundColor Gray
        }
    }

    # ~/.cargo/bin kontrolü
    $cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
    if (Test-Path $cargoBin) {
        $cargoFiles = @(
            (Join-Path $cargoBin "clrinf-codegen.exe"),
            (Join-Path $cargoBin "clrinf.exe")
        )
        foreach ($file in $cargoFiles) {
            if (Test-Path $file) {
                Remove-Item -Path $file -Force -ErrorAction SilentlyContinue
                Write-Host "✓ Silindi: $file" -ForegroundColor Gray
            }
        }
    }

    # PATH temizliği
    $scope = if ($System) { "Machine" } else { "User" }
    Remove-PathEntry -TargetDirectory $InstallDir -Scope $scope

    # Boş kalan dizini temizle
    if ((Test-Path $InstallDir) -and ((Get-ChildItem -Path $InstallDir -Force | Measure-Object).Count -eq 0)) {
        Remove-Item -Path $InstallDir -Force -Recurse -ErrorAction SilentlyContinue
    }

    Write-Host "✅ clrinf başarıyla kaldırıldı." -ForegroundColor Green
    exit 0
}

# ==============================================================================
# KURULUM AKIŞI
# ==============================================================================
Write-Host "🚀 clrinf Windows Kurulum Motoru başlatılıyor..." -ForegroundColor Cyan

if ($System -and -not (Test-IsAdmin)) {
    Write-Host "❌ Sistem geneline kurulum (-System) için PowerShell Yönetici (Administrator) olarak çalıştırılmalıdır." -ForegroundColor Red
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
            Write-Host "✓ Yerel olarak derlenmiş ikili dosya bulundu: $TargetBin" -ForegroundColor Green
            break
        }
    }

    # Yerel binary yoksa, GitHub Releases üzerinden indirmeyi dene
    if (-not $TargetBin) {
        $platform = Get-PlatformTriple
        Write-Host "🌐 GitHub Releases üzerinden önceden derlenmiş paket aranıyor ($platform)..." -ForegroundColor Blue

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
                    Write-Host "✓ Önceden derlenmiş paket indirildi. Arşiv açılıyor..." -ForegroundColor Green
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
                # Diğer URL veya kaynak koddan derleme fallback'ine geç
            }
        }

        if (-not $TargetBin) {
            Write-Host "ℹ️  GitHub Release paketi bulunamadı (henüz yayınlanmamış veya ağ erişilemiyor)." -ForegroundColor Yellow
        }
    }
}

# Fallback: İkili dosya bulunamadıysa veya -Build parametresi verildiyse kaynak koddan derle
if (-not $TargetBin -or $Build) {
    $cargoToml = Join-Path $CodegenDir "Cargo.toml"
    if ((Test-Path $CodegenDir) -and (Test-Path $cargoToml)) {
        Write-Host "📦 clrinf-codegen yerel kaynak koddan derleniyor (cargo build --release)..." -ForegroundColor Blue

        if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
            Write-Host "❌ 'cargo' bulunamadı! İkili paket bulunamadığı için derleme adına Rust gereklidir." -ForegroundColor Red
            Write-Host "👉 https://rustup.rs adresinden veya 'winget install Rustlang.Rustup' komutuyla Rust kurabilirsiniz." -ForegroundColor Yellow
            exit 1
        }

        Push-Location $CodegenDir
        try {
            cargo build --release
            if ($LASTEXITCODE -ne 0) {
                Write-Host "❌ Derleme sırasında bir hata oluştu." -ForegroundColor Red
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
        Write-Host "❌ Kurulum için uygun bir ikili dosya bulunamadı ve yerel kaynak kod dizini mevcut değil." -ForegroundColor Red
        Write-Host "👉 Depoyu kaynak koddan klonlayarak kurmak için:" -ForegroundColor Yellow
        Write-Host "   git clone --recurse-submodules https://github.com/$GithubRepo.git"
        Write-Host "   cd clrinf; .\install.ps1 -Build"
        exit 1
    }
}

if (-not $TargetBin -or -not (Test-Path $TargetBin)) {
    Write-Host "❌ İkili dosya doğrulanamadı: $TargetBin" -ForegroundColor Red
    exit 1
}

# 2. Hedef dizini hazırla ve kopyala
Write-Host "📁 Hedef dizin hazırlanıyor: $InstallDir" -ForegroundColor Blue
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

$destCodegen = Join-Path $InstallDir "clrinf-codegen.exe"
$destClrinf = Join-Path $InstallDir "clrinf.exe"

Copy-Item -Path $TargetBin -Destination $destCodegen -Force
Copy-Item -Path $TargetBin -Destination $destClrinf -Force
Write-Host "✓ İkili dosyalar kopyalandı:" -ForegroundColor Green
Write-Host "    $destCodegen" -ForegroundColor Gray
Write-Host "    $destClrinf" -ForegroundColor Gray

# ~/.cargo/bin mevcutsa oraya da kopyala (kolay erişim için)
$cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
if ((Test-Path $cargoBin) -and ($InstallDir.TrimEnd('\') -ne $cargoBin.TrimEnd('\'))) {
    try {
        Copy-Item -Path $TargetBin -Destination (Join-Path $cargoBin "clrinf-codegen.exe") -Force -ErrorAction SilentlyContinue
        Copy-Item -Path $TargetBin -Destination (Join-Path $cargoBin "clrinf.exe") -Force -ErrorAction SilentlyContinue
        Write-Host "✓ Cargo ortamı için eşlendi: $cargoBin" -ForegroundColor Gray
    } catch {
        # Cargo dizinine kopyalama başarısız olursa devam et
    }
}

# 3. PATH Kontrolü ve Yapılandırması
$scope = if ($System) { "Machine" } else { "User" }
Add-PathEntry -TargetDirectory $InstallDir -Scope $scope

# Mevcut PowerShell oturumunun PATH değişkenini güncelle
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
Write-Host "🎉 clrinf CLI ve araçları başarıyla kuruldu!" -ForegroundColor Green
Write-Host "  📍 İkili Dosya: $destCodegen"
Write-Host "  🔗 Kısayol:     $destClrinf"
Write-Host ""
Write-Host "Hızlı Doğrulama Komutları:" -ForegroundColor Cyan
Write-Host "  clrinf --version"
Write-Host "  clrinf catalog"
Write-Host "  clrinf docs --lang csharp"
Write-Host "  clrinf mcp"
Write-Host ""
