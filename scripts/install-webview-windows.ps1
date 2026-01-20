#Requires -Version 5.1
<#
.SYNOPSIS
    Install Microsoft Edge WebView2 Runtime for Communitas on Windows.

.DESCRIPTION
    This script downloads and installs the Microsoft Edge WebView2 Runtime,
    which is required by Dioxus/Wry for rendering the Communitas UI.

    WebView2 may already be installed if:
    - You have Microsoft Edge browser installed
    - Windows 11 (includes WebView2 by default)
    - An application previously installed it

.NOTES
    Author: Communitas Team
    Requires: PowerShell 5.1 or later, Administrator privileges (for system-wide install)

.EXAMPLE
    .\install-webview-windows.ps1
    Downloads and installs WebView2 for all users (requires admin).

.EXAMPLE
    .\install-webview-windows.ps1 -UserInstall
    Installs WebView2 for the current user only (no admin required).
#>

param(
    [switch]$UserInstall,
    [switch]$Silent,
    [switch]$Force
)

$ErrorActionPreference = "Stop"

function Write-Log {
    param([string]$Message, [string]$Level = "INFO")
    $color = switch ($Level) {
        "INFO"  { "Green" }
        "WARN"  { "Yellow" }
        "ERROR" { "Red" }
        default { "White" }
    }
    Write-Host "[$Level] " -ForegroundColor $color -NoNewline
    Write-Host $Message
}

function Test-WebView2Installed {
    # Check registry for WebView2 installation
    $registryPaths = @(
        "HKLM:\SOFTWARE\Microsoft\EdgeWebView",
        "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeWebView",
        "HKCU:\SOFTWARE\Microsoft\EdgeWebView"
    )

    foreach ($path in $registryPaths) {
        if (Test-Path $path) {
            try {
                $version = (Get-ItemProperty -Path $path -ErrorAction SilentlyContinue).pv
                if ($version) {
                    return $version
                }
            } catch {
                # Continue checking other paths
            }
        }
    }

    # Check file system as fallback
    $programFiles = @($env:ProgramFiles, ${env:ProgramFiles(x86)}, $env:LOCALAPPDATA)
    foreach ($base in $programFiles) {
        if (-not $base) { continue }
        $webviewPath = Join-Path $base "Microsoft\EdgeWebView\Application"
        if (Test-Path $webviewPath) {
            $versions = Get-ChildItem -Path $webviewPath -Directory -ErrorAction SilentlyContinue |
                        Where-Object { $_.Name -match '^\d+\.' }
            if ($versions) {
                return $versions[0].Name
            }
        }
    }

    return $null
}

function Get-WebView2Bootstrapper {
    param([string]$DownloadPath)

    $bootstrapperUrl = "https://go.microsoft.com/fwlink/p/?LinkId=2124703"
    Write-Log "Downloading WebView2 bootstrapper..."

    try {
        # Use TLS 1.2 for HTTPS
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

        $webClient = New-Object System.Net.WebClient
        $webClient.DownloadFile($bootstrapperUrl, $DownloadPath)

        if (Test-Path $DownloadPath) {
            Write-Log "Download complete: $DownloadPath"
            return $true
        }
    } catch {
        Write-Log "Download failed: $($_.Exception.Message)" -Level "ERROR"
        return $false
    }

    return $false
}

function Install-WebView2 {
    param(
        [string]$InstallerPath,
        [bool]$PerUser,
        [bool]$SilentMode
    )

    $arguments = @()

    if ($SilentMode) {
        $arguments += "/silent"
    }

    if ($PerUser) {
        # Per-user install doesn't require elevation
        $arguments += "/install"
    } else {
        # Check for admin rights for system-wide install
        $isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
        if (-not $isAdmin) {
            Write-Log "System-wide installation requires administrator privileges." -Level "WARN"
            Write-Log "Please run PowerShell as Administrator, or use -UserInstall for per-user installation."
            return $false
        }
    }

    Write-Log "Running WebView2 installer..."
    try {
        $process = Start-Process -FilePath $InstallerPath -ArgumentList $arguments -Wait -PassThru
        return $process.ExitCode -eq 0
    } catch {
        Write-Log "Installation failed: $($_.Exception.Message)" -Level "ERROR"
        return $false
    }
}

function Main {
    Write-Host "========================================"
    Write-Host "  Communitas WebView2 Installer"
    Write-Host "========================================"
    Write-Host

    # Check if already installed
    $existingVersion = Test-WebView2Installed
    if ($existingVersion -and -not $Force) {
        Write-Log "WebView2 is already installed (version: $existingVersion)"
        Write-Log "Use -Force to reinstall."
        return
    }

    if ($existingVersion) {
        Write-Log "WebView2 version $existingVersion found, reinstalling..." -Level "WARN"
    }

    # Create temp directory for download
    $tempDir = Join-Path $env:TEMP "CommunutasWebView2Install"
    if (-not (Test-Path $tempDir)) {
        New-Item -ItemType Directory -Path $tempDir -Force | Out-Null
    }

    $bootstrapperPath = Join-Path $tempDir "MicrosoftEdgeWebview2Setup.exe"

    # Download the bootstrapper
    if (-not (Get-WebView2Bootstrapper -DownloadPath $bootstrapperPath)) {
        Write-Log "Failed to download WebView2 bootstrapper" -Level "ERROR"
        exit 1
    }

    # Install
    $success = Install-WebView2 -InstallerPath $bootstrapperPath -PerUser $UserInstall -SilentMode $Silent

    # Cleanup
    if (Test-Path $tempDir) {
        Remove-Item -Path $tempDir -Recurse -Force -ErrorAction SilentlyContinue
    }

    if ($success) {
        # Verify installation
        $installedVersion = Test-WebView2Installed
        if ($installedVersion) {
            Write-Host
            Write-Log "WebView2 installation complete (version: $installedVersion)"
            Write-Log "You can now run Communitas."
        } else {
            Write-Log "Installation completed but verification failed. Please restart and try again." -Level "WARN"
        }
    } else {
        Write-Log "WebView2 installation failed." -Level "ERROR"
        Write-Host
        Write-Host "You can manually download WebView2 from:"
        Write-Host "https://developer.microsoft.com/en-us/microsoft-edge/webview2/"
        exit 1
    }
}

Main
