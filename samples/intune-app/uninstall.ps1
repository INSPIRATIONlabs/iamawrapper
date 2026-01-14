<#
.SYNOPSIS
    Contoso Configuration Tool - Uninstallation Script

.DESCRIPTION
    This script removes the Contoso Configuration Tool from Windows devices.

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
$InstallPath = "$env:ProgramFiles\Contoso\ConfigTool"
$LogPath = "$env:ProgramData\Contoso\Logs"
$LogFile = "$LogPath\uninstall.log"

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

# Main uninstallation
try {
    Write-Log "Starting uninstallation of $AppName"

    # Remove application files
    if (Test-Path $InstallPath) {
        Write-Log "Removing application files from: $InstallPath"
        Remove-Item -Path $InstallPath -Recurse -Force
    }

    # Remove parent directory if empty
    $parentPath = "$env:ProgramFiles\Contoso"
    if ((Test-Path $parentPath) -and ((Get-ChildItem $parentPath | Measure-Object).Count -eq 0)) {
        Remove-Item -Path $parentPath -Force
    }

    # Remove registry entries
    $uninstallKey = "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\ContosoConfigTool"
    if (Test-Path $uninstallKey) {
        Write-Log "Removing registry entries..."
        Remove-Item -Path $uninstallKey -Recurse -Force
    }

    Write-Log "Uninstallation completed successfully"
    exit 0
}
catch {
    Write-Log "ERROR: Uninstallation failed - $_"
    exit 1
}
