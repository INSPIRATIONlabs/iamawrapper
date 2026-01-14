<#
.SYNOPSIS
    Contoso Configuration Tool - Installation Script

.DESCRIPTION
    This script installs the Contoso Configuration Tool on Windows devices.
    Designed for deployment via Microsoft Intune.

.NOTES
    Version:        1.0.0
    Author:         Contoso IT
    Creation Date:  2026-01-14
#>

param(
    [switch]$Silent
)

$ErrorActionPreference = "Stop"

# Configuration
$AppName = "Contoso Configuration Tool"
$Version = "1.0.0"
$InstallPath = "$env:ProgramFiles\Contoso\ConfigTool"
$LogPath = "$env:ProgramData\Contoso\Logs"
$LogFile = "$LogPath\install.log"

# Logging function
function Write-Log {
    param([string]$Message)
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    $logMessage = "$timestamp - $Message"

    if (-not (Test-Path $LogPath)) {
        New-Item -Path $LogPath -ItemType Directory -Force | Out-Null
    }

    Add-Content -Path $LogFile -Value $logMessage

    if (-not $Silent) {
        Write-Host $logMessage
    }
}

# Main installation
try {
    Write-Log "Starting installation of $AppName v$Version"

    # Create installation directory
    if (-not (Test-Path $InstallPath)) {
        Write-Log "Creating installation directory: $InstallPath"
        New-Item -Path $InstallPath -ItemType Directory -Force | Out-Null
    }

    # Copy application files
    Write-Log "Copying application files..."
    $ScriptPath = Split-Path -Parent $MyInvocation.MyCommand.Path
    Copy-Item -Path "$ScriptPath\config.json" -Destination $InstallPath -Force

    # Create the main executable (PowerShell script as entry point)
    $exeContent = @"
# Contoso Configuration Tool
Write-Host "Contoso Configuration Tool v$Version"
Write-Host "Configuration loaded from: $InstallPath\config.json"
"@
    Set-Content -Path "$InstallPath\contoso-tool.ps1" -Value $exeContent

    # Create uninstall registry entry
    $uninstallKey = "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\ContosoConfigTool"
    Write-Log "Creating registry entries..."

    if (-not (Test-Path $uninstallKey)) {
        New-Item -Path $uninstallKey -Force | Out-Null
    }

    Set-ItemProperty -Path $uninstallKey -Name "DisplayName" -Value $AppName
    Set-ItemProperty -Path $uninstallKey -Name "DisplayVersion" -Value $Version
    Set-ItemProperty -Path $uninstallKey -Name "Publisher" -Value "Contoso Corporation"
    Set-ItemProperty -Path $uninstallKey -Name "InstallLocation" -Value $InstallPath
    Set-ItemProperty -Path $uninstallKey -Name "UninstallString" -Value "powershell.exe -ExecutionPolicy Bypass -File `"$InstallPath\uninstall.ps1`""
    Set-ItemProperty -Path $uninstallKey -Name "NoModify" -Value 1
    Set-ItemProperty -Path $uninstallKey -Name "NoRepair" -Value 1

    # Copy uninstall script
    if (Test-Path "$ScriptPath\uninstall.ps1") {
        Copy-Item -Path "$ScriptPath\uninstall.ps1" -Destination $InstallPath -Force
    }

    Write-Log "Installation completed successfully"
    exit 0
}
catch {
    Write-Log "ERROR: Installation failed - $_"
    exit 1
}
