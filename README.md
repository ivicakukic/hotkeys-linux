# HotKeys Linux

A Linux port of the Windows HotKeys application for mapping keyboard shortcuts/actions to numeric keypad keys [1-9].

## Table of Contents

- [Usage](#usage)
- [Installation & Setup](#installation--setup)
  - [Prerequisites](#prerequisites)
  - [Install from Debian Package](#install-from-debian-package)
  - [Build from Source](#build-from-source)
  - [Input Device Permissions (Manual Setup)](#input-device-permissions-manual-setup)
    - [Reverting Permissions](#reverting-input-device-permissions)
  - [Setting up Global Shortcuts (GNOME/Wayland)](#setting-up-global-shortcuts-gnomewayland)
    - [Single Profile Setup](#single-profile-setup)
    - [Multiple Profile Setup (Recommended)](#multiple-profile-setup-recommended)
  - [Configuration Files](#configuration-files)
- [Configuration](#configuration)
  - [Configuration Structure](#configuration-structure)
  - [File Includes System](#file-includes-system)
  - [Profile System](#profile-system)
  - [Boards and Application Detection](#boards-and-application-detection)
  - [Pads and Actions](#pads-and-actions)
  - [Action Types](#action-types)
  - [Modifier Key System](#modifier-key-system)
  - [Board Navigation](#board-navigation)
  - [Application Settings](#application-settings)
  - [Visual Customization](#visual-customization)
- [Platform Limitations](#platform-limitations)
  - [Wayland Security Model](#wayland-security-model)
  - [X11 Performance Issues](#x11-performance-issues)
- [Development](#development)
  - [Running the Application](#running-the-application)
  - [Testing](#testing)
  - [Debian Packaging](#debian-packaging)
- [TODO](#todo)
- [Contributing](#contributing)
- [License](#license)


## Usage

The application is triggered by a global keyboard shortcut and works as follows:

- User hits the configured global shortcut (e.g., `Ctrl+Alt+NumPad_0`)
- HotKeys starts and attempts to detect the currently active application
- If a board is configured for the detected app:
  - A 3x3 board is displayed
  - User selects an action using numeric keys [1-9]
  - Board closes automatically after action or timeout
- User can also close the board by pressing any other key

## Installation & Setup

### Prerequisites
- Linux system with X11 or Wayland
- GTK4 development libraries
- Rust toolchain

### Install from Debian Package

If you have the `.deb` package available:
```bash
# Install build dependencies (one-time setup)
sudo apt install debhelper devscripts build-essential
sudo apt install libgtk-4-dev libgdk-pixbuf-2.0-dev libcairo2-dev
sudo apt install libpango1.0-dev libatk1.0-dev pkg-config
sudo apt install libx11-dev x11-utils

# Build the package (from project root)
dpkg-buildpackage -b -uc -us -d

# Install the generated package
sudo dpkg -i ../hotkeys_0.1.0-1_amd64.deb
sudo apt-get install -f  # Fix any missing runtime dependencies
```

### Build from Source
```bash
git clone <repository-url>
cd hotkeys
cargo build --release
```

### Input Device Permissions (Manual Setup)

If installing manually (not via .deb package), you'll need to configure udev permissions for input device access:

```bash
# Create udev rule for input device access
sudo tee /etc/udev/rules.d/99-uinput.rules << EOF
KERNEL=="uinput", MODE="0660", GROUP="input"
EOF

# Reload udev rules and add your user to input group
sudo udevadm control --reload-rules
sudo usermod -a -G input $USER
sudo modprobe uinput

# Log out and log back in (or reboot) for group membership to take effect
```

**Why this is needed:** HotKeys uses the Linux `uinput` subsystem to simulate keyboard input. This requires access to `/dev/uinput` and membership in the `input` group. The .deb package handles this automatically during installation.

#### Reverting Permissions

```bash
# Remove the udev rule
sudo rm /etc/udev/rules.d/99-uinput.rules

# Remove user from input group
sudo deluser $USER input

# Reload udev rules
sudo udevadm control --reload-rules

# Remove uinput module (optional - will be loaded again on demand)
sudo modprobe -r uinput
```

Log out/in (or reboot) in order for the group membership change to take effect.

### Setting up Global Shortcuts (GNOME/Wayland)

#### Single Profile Setup
1. Open GNOME Settings → Keyboard → Keyboard Shortcuts → Custom Shortcuts
2. Add a new shortcut:
   - **Name**: `hotkeys`
   - **Command**: `hotkeys` (if installed via .deb) or `/path/to/hotkeys` (if built from source)
   - **Shortcut**: `Ctrl+Alt+Insert` (or your preferred combination)

#### Multiple Profile Setup (Recommended)
Set up different shortcuts for different contexts using profiles:

```bash
# Configure multiple keybindings
dconf write /org/gnome/settings-daemon/plugins/media-keys/custom-keybindings "['@as ['/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom0/', '/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom1/', '/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom2/']']"

# IDEs profile - Ctrl+Alt+Insert
dconf write /org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom0/name "'hotkeys ides'"
dconf write /org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom0/command "'hotkeys --profile ides'"
dconf write /org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom0/binding "'<Control><Alt>KP_Insert'"

# Browsers profile - Ctrl+Alt+End
dconf write /org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom1/name "'hotkeys browsers'"
dconf write /org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom1/command "'hotkeys --profile browsers'"
dconf write /org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom1/binding "'<Control><Alt>KP_End'"

# Default profile - Ctrl+Alt+Enter
dconf write /org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom2/name "'hotkeys default'"
dconf write /org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom2/command "'hotkeys --profile default'"
dconf write /org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom2/binding "'<Control><Alt>KP_Enter'"
```

### Configuration Files
The application uses automatic configuration resolution:

**Development:** Uses local `resources/` directory

**Production:** Resolves from 2 directories in this order
1. `--config_dir <path>` if specified (custom), else `~/.config/hotkeys/` (default)
2. `/usr/share/hotkeys/` (system-wide resources)

**Key Configuration Files:**
- `settings.json` - Main configuration with boards, profiles, and styling
- `log.toml` - Logging configuration

## Usage

The application supports multiple execution modes:

```bash
# Default mode (GTK with automatic config resolution)
hotkeys

# Show help and usage
hotkeys help

# Validate settings.json (dry-run)
hotkeys validate-settings

# Test input system
hotkeys input-test

# Use specific profile
hotkeys --profile browsers

# Use custom config directory
hotkeys --config_dir /path/to/config

# Combined options
hotkeys --profile ides --config_dir /custom/config
```

**Options:**
- `--profile <name>`: Use specific profile (e.g., `ides`, `browsers`, `default`)
- `--config_dir <path>`: Use specified config directory (overrides automatic resolution)
- Default profile: `default`
- Default config: Automatic resolution (see Configuration Files section)

## Configuration

The application uses a JSON-based configuration system with automatic file includes and profile support.

### Configuration Structure

**Main Files:**
- `settings.json` - Main configuration with profiles, boards, and application settings
- `log.toml` - Logging configuration

**File Resolution Order:**
1. `--config_dir <path>` (if specified) or `~/.config/hotkeys/` (user config)
2. `/usr/share/hotkeys/` (system resources)

### File Includes System

The configuration supports modular organization through the `includes` mechanism, allowing you to split configuration across multiple files:

```json
{
  "includes": [
    "settings.styling.json",
    "settings.keyboard.json",
    "mine/settings.board.code.json",
    "mine/settings.board.chrome.json"
  ],
  "profiles": [...],
  "boards": [...]
}
```

**How it works:**
- Include files are resolved relative to the main `settings.json` location
- All included files are merged into the main configuration
- Arrays are concatenated, objects are merged (included files override main settings)
- Nested includes are not supported (only main file can include others)

### Profile System

**Profiles** group related boards for specific workflows, enabling context-aware shortcuts:

```json
{
  "profiles": [
    {
      "name": "ides",
      "boards": ["code", "idea64", "sublime_text"],
      "default": "code"
    },
    {
      "name": "browsers",
      "boards": ["chrome", "firefox"],
      "default": "chrome"
    },
    {
      "name": "default",
      "boards": ["hotkeys"],
      "default": "hotkeys"
    }
  ]
}
```

**Key Benefits:**
- **Context Separation**: Keep IDE boards separate from browser boards
- **Multiple Global Shortcuts**: Different shortcuts for different workflows
- **Scoped Detection**: Only boards in active profile are considered for app matching
- **Default Fallback**: Each profile defines fallback board when no app detected

**Real-World Usage:**
- `Ctrl+Alt+Insert` → IDE profile (VS Code, IntelliJ, Sublime)
- `Ctrl+Alt+End` → Browser profile (Chrome, Firefox)
- `Ctrl+Alt+Enter` → Default profile (HotKeys home, system commands)

### Boards and Application Detection

Boards define application-specific shortcut mappings. Each board targets a specific application:

```json
{
  "boards": [
    {
      "title": "VS Code",
      "name": "code",
      "icon": "mine/code.svg",
      "color_scheme": "Blue",
      "detection": {
        "ps": "code"
      },
      "base_pads": "code"
    }
  ]
}
```

**Detection Methods:**
- `"ps": "process_name"` - Match by process name (case-insensitive substring)
- `"xprop": "window_class"` - Match by X11 window class (X11 only)
- `"none"` - Manual selection only (for home boards and sub-boards)

### Pads and Actions

Pads define the 3x3 grid layout. Each pad can contain actions and/or navigation:

```json
{
  "padsets": [
    {
      "name": "code",
      "items": [
        {
          "header": "F12",
          "text": "Go to Definition",
          "icon": "mine/goto.svg",
          "actions": [
            {"Shortcut": "F12"}
          ]
        },
        {
          "header": "Ctrl Shift P",
          "text": "Command Palette",
          "actions": [
            {"Shortcut": "Ctrl Shift P"}
          ],
          "board": "code/commands"
        }
      ]
    }
  ]
}
```

### Action Types

HotKeys supports multiple action types that can be combined in sequences:

| Action | Description | Example |
|--------|-------------|---------|
| **Shortcut** | Send keyboard shortcuts | `{"Shortcut": "Ctrl C"}` |
| **Text** | Send arbitrary text | `{"Text": "Hello World"}` |
| **Line** | Send text + ENTER | `{"Line": "git status"}` |
| **Pause** | Wait milliseconds | `{"Pause": 500}` |
| **Command** | Execute shell command | `{"Command": "docker start postgres"}` |
| **OpenUrl** | Open URL in browser | `{"OpenUrl": "https://github.com"}` |

**Action Sequences:**
```json
{
  "header": "New Terminal Tab",
  "actions": [
    {"Shortcut": "Ctrl Shift T"},
    {"Pause": 100},
    {"Line": "cd ~/projects"},
    {"Line": "ls -la"}
  ]
}
```

**Shortcut Syntax:**
- Single keys: `"Ctrl C"`, `"Alt F4"`, `"F12"`
- Chord sequences: `"Ctrl K + Ctrl B"` (VS Code style)
- Special characters: `"Ctrl Shift '+'"`

### Modifier Key System

**Dynamic Board Switching**: Hold modifier keys to instantly change board content:

```json
{
  "name": "chrome",
  "base_pads": "chrome",
  "modifier_pads": {
    "Ctrl": "chrome/browser-switch",
    "Ctrl+Shift": "chrome/dev-tools"
  }
}
```

**Supported Modifiers:**
- `Ctrl`, `Shift`, `Alt`, `Super`
- Combinations: `Ctrl+Shift`, `Ctrl+Alt`, `Alt+Super`, etc.

**Use Cases:**
- **Browser Switching**: Hold Ctrl to see Chrome/Firefox options
- **IDE Submenus**: Ctrl+5 shows bookmark submenu instead of bookmark toggle
- **Power User Workflows**: Advanced actions hidden behind modifier combinations

### Board Navigation

Create hierarchical board structures using the `board` field:

```json
{
  "header": "Settings",
  "text": "App Settings",
  "actions": [
    {"Shortcut": "Ctrl Comma"}
  ],
  "board": "code/settings"
}
```

**Navigation Rules:**
- Actions execute first, then navigation occurs

### Application Settings

Global application configuration:

```json
{
  "timeout": 5,
  "feedback": 2,
  "delay": 1,
  "keyboard_layout": "default",
  "layout": {
    "width": 883,
    "height": 597,
    "window_style": "Taskbar"
  }
}
```

**Settings Reference:**
- `timeout`: Auto-close seconds (integer)
- `feedback`: Visual feedback duration (integer)
- `delay`: Input delay between actions (integer)
- `keyboard_layout`: Active layout name for character mapping
- `window_style`: `"Window"` (with title bar) or `"Taskbar"` (borderless)

### Visual Customization

**Color Schemes:**
```json
{
  "color_schemes": [
    {
      "name": "Blue",
      "opacity": 0.9,
      "background": "#00007f",
      "foreground1": "#5454a9",
      "foreground2": "#dbdbec"
    }
  ]
}
```

**Icon Support:**
HotKeys supports both PNG and SVG icons. For SVG icons to properly integrate with the color scheme theming:

```svg
<!-- Add CSS classes to SVG elements for automatic color theming -->
<path class="board-f" d="..."/>      <!-- Fill with foreground2 color -->
<path class="board-s" d="..."/>      <!-- Stroke with foreground2 color -->
<rect class="board-sf" d="..."/>     <!-- Both stroke and fill with foreground2 color -->
```

**Available CSS Classes:**
- `.board-f` - Applies `fill` with the active color scheme's `foreground2` color
- `.board-s` - Applies `stroke` with the active color scheme's `foreground2` color
- `.board-sf` - Applies both `stroke` and `fill` with the active color scheme's `foreground2` color

This allows SVG icons to automatically adapt to different color schemes. PNG icons are displayed as-is without color modification.

**Text Styles:**
```json
{
  "text_styles": [
    {
      "name": "default",
      "header_font": "Impact Bold 24",
      "pad_header_font": "Consolas 14",
      "pad_text_font": "Arial Bold 16",
      "pad_id_font": "Impact Bold 16"
    }
  ]
}
```

**Keyboard Layouts** (for non-US keyboards):
```json
{
  "keyboard_layouts": [
    {
      "name": "croatian",
      "mappings": {
        "š": "[",
        "Š": "{",
        "đ": "]"
      }
    }
  ]
}
```

## Platform Limitations

### Wayland Security Model
Due to Wayland's security-first design:
- **Global hotkeys** only work with Xwayland applications
- **Active window detection** fails for native Wayland applications
- These limitations are by design and affect all automation tools

### X11 Performance Issues
While X11 provides full functionality:
- Higher latency and sluggish UI interactions
- Animation conflicts between board and application transitions
- Less efficient rendering compared to Wayland

## Development

### Running the Application
```bash
# Default mode (GTK with automatic config resolution)
cargo run

# With debug logging
RUST_LOG=debug cargo run

# Validate settings.json (dry-run)
cargo run validate-settings

# Test input system
cargo run input-test

# With specific options
cargo run -- --profile browsers --config_dir /custom/config
```

### Testing
```bash
cargo test
cargo check
```

### Debian Packaging

The project includes complete Debian packaging support:

```bash
# Clean build artifacts
debian/rules clean
# or
rm -rf debian/cargo_home debian/hotkeys debian/debhelper-build-stamp debian/files debian/*.substvars debian/*.debhelper debian/.debhelper debian/tmp

# Build package (requires build dependencies installed)
dpkg-buildpackage -b -uc -us -d

# Package installs to system locations:
# - Binary: /usr/bin/hotkeys
# - Resources: /usr/share/hotkeys/
# - Desktop entry: /usr/share/applications/
# - Documentation: /usr/share/doc/hotkeys/
```

**Package Management:**
- Source files in `debian/` are committed to git
- Generated files (build artifacts) are gitignored
- Update `debian/changelog` for new releases

## TODO

- [ ] Multi-monitor support
- [ ] RPM packaging for Red Hat/Fedora systems
- [ ] Global hotkey registration improvements
- [ ] Wayland protocol extension support (experimental)

## Contributing

This project is a Linux port focusing on desktop automation while respecting platform security models. Contributions are welcome, especially for:
- Performance improvements on X11
- Alternative solutions for Wayland limitations
- Enhanced configuration management
- Better user experience features

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.