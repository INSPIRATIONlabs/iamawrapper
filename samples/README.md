# Sample Applications

This directory contains sample applications that demonstrate how to use iamawrapper to create deployment packages.

## Directory Structure

```
samples/
├── intune-app/          # Windows application for Microsoft Intune
│   ├── install.ps1      # Installation script (setup file)
│   ├── uninstall.ps1    # Uninstallation script
│   └── config.json      # Application configuration
│
└── macos-app/           # macOS application bundle
    ├── ContosoTool.app/ # Application bundle
    │   └── Contents/
    │       ├── Info.plist
    │       └── MacOS/
    │           └── contoso-tool
    └── scripts/         # Installation scripts
        ├── preinstall
        └── postinstall
```

---

## Creating an Intune Package (.intunewin)

### Step 1: Navigate to the repository root

```bash
cd /path/to/iamawrapper
```

### Step 2: Create the package

```bash
# Using the CLI
iamawrapper intune create -c samples/intune-app -s install.ps1 -o output

# Or with full flags
iamawrapper intune create --content samples/intune-app --setup install.ps1 --output output
```

### Step 3: Verify the output

```bash
ls -la output/install.intunewin
```

### Step 4: Upload to Microsoft Intune

1. Sign in to the [Microsoft Intune admin center](https://intune.microsoft.com)
2. Navigate to **Apps** > **All apps** > **Add**
3. Select **Windows app (Win32)** as the app type
4. Upload `output/install.intunewin`
5. Configure the app:
   - **Name**: Contoso Configuration Tool
   - **Publisher**: Contoso Corporation
   - **Install command**: `powershell.exe -ExecutionPolicy Bypass -File install.ps1`
   - **Uninstall command**: `powershell.exe -ExecutionPolicy Bypass -File uninstall.ps1`
   - **Detection rule**: Registry key exists `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\ContosoConfigTool`

### Extracting an existing Intune package

To extract the contents of an existing `.intunewin` file:

```bash
iamawrapper intune extract -i output/install.intunewin -o extracted
```

---

## Creating a macOS Package (.pkg)

### Step 1: Navigate to the repository root

```bash
cd /path/to/iamawrapper
```

### Step 2: Make scripts executable (if needed)

```bash
chmod +x samples/macos-app/scripts/preinstall
chmod +x samples/macos-app/scripts/postinstall
chmod +x samples/macos-app/ContosoTool.app/Contents/MacOS/contoso-tool
```

### Step 3: Create the package

```bash
iamawrapper macos pkg \
  -c samples/macos-app/ContosoTool.app \
  -o output/ContosoTool.pkg \
  --identifier com.contoso.tool \
  --version 1.0.0 \
  --install-location /Applications \
  --scripts samples/macos-app/scripts
```

### Step 4: Verify the output

```bash
ls -la output/ContosoTool.pkg
```

### Step 5: Install the package (on macOS)

```bash
# Interactive installation (double-click the .pkg file)
open output/ContosoTool.pkg

# Command-line installation (requires admin privileges)
sudo installer -pkg output/ContosoTool.pkg -target /
```

### Step 6: Deploy via MDM

The `.pkg` file can be deployed through:
- **Apple Business Manager** / **Apple School Manager**
- **Jamf Pro**
- **Microsoft Intune** (as a macOS LOB app)
- **Kandji**
- **Mosyle**
- Other MDM solutions that support macOS flat packages

---

## Package Comparison

| Feature | Intune (.intunewin) | macOS (.pkg) |
|---------|---------------------|--------------|
| Target OS | Windows | macOS |
| Encryption | AES-256-CBC | None |
| Scripts | PowerShell | Shell (bash) |
| Install method | Intune agent | macOS Installer |
| Silent install | `-Silent` flag | `installer -pkg` |

---

## Customizing the Samples

### For Intune packages

1. Replace the scripts and files in `samples/intune-app/` with your application
2. Ensure `install.ps1` handles your installation logic
3. Update `config.json` with your application settings
4. Create detection rules based on your app's installation footprint

### For macOS packages

1. Replace `ContosoTool.app` with your actual application bundle
2. Update `Info.plist` with your app's metadata
3. Modify `preinstall` and `postinstall` scripts as needed
4. Use the correct bundle identifier with `--identifier`

---

## Troubleshooting

### Intune package issues

- **Upload fails**: Ensure the Detection.xml format is correct (iamawrapper handles this automatically)
- **Installation fails**: Check the install.ps1 script for errors
- **Detection fails**: Verify your detection rules match what the installer creates

### macOS package issues

- **Package won't install**: Check that scripts have executable permissions
- **App won't launch**: Verify Info.plist has correct CFBundleExecutable
- **Gatekeeper blocks**: The package may need to be signed for distribution
