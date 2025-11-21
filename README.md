# Vitray Widget

A glassmorphic desktop widget for Debian/Ubuntu with system monitoring and terminal access.

## Features

- **2x2 Monitoring Grid**: CPU, GPU, RAM, and Network stats with color-coded status
- **Tabbed Terminal**: Multiple terminal sessions in one widget
- **Shortcuts System**: Save and run commands with one click
- **Settings**: Themes, toggles, and auto-start configuration
- **Glassmorphic Design**: Semi-transparent with blur support

## Installation

### APT Repository (Recommended)

Add the repository and install:

```bash
# Add repository (replace zacxxx with your GitHub username)
echo "deb [trusted=yes] https://zacxxx.github.io/vitray-widget stable main" | sudo tee /etc/apt/sources.list.d/vitray-widget.list

# Update and install
sudo apt update
sudo apt install vitray-widget
```

Updates will be automatically available via `sudo apt update && sudo apt upgrade`.

### Manual Installation

Download the latest `.deb` from [Releases](https://github.com/zacxxx/vitray-widget/releases):

```bash
sudo apt install ./vitray-widget_0.2.0-1_amd64.deb
```

## Usage

### Launch
- From application menu: "Vitray Widget"
- From terminal: `vitray-widget`

### Adding Shortcuts
```bash
vitray-widget --shortcut "htop" "System Monitor"
vitray-widget --shortcut "git status" "Git Status"
```

### Settings & Shortcuts
- **Right-click** the widget to access settings or shortcuts panel
- Change theme, toggle widgets, enable auto-start
- Run shortcuts directly in the terminal

## Building from Source

### Prerequisites
```bash
sudo apt install libgtk-4-dev libvte-2.91-gtk4-dev pkg-config
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Build
```bash
git clone https://github.com/zacxxx/vitray-widget.git
cd vitray-widget
cargo build --release
./target/release/vitray-widget
```

### Create Package
```bash
cargo install cargo-deb
cargo deb
```

## Configuration

- Settings: `~/.config/vitray-widget/settings.json`
- Shortcuts: `~/.config/vitray-widget/shortcuts.json`
- CSS: `/usr/share/vitray-widget/style.css`

## Troubleshooting

- **GPU Stats**: Requires `nvidia-smi` for NVIDIA GPUs
- **Transparency**: Requires compositor support (GNOME, KDE, etc.)
- **Blur**: Configure compositor to blur windows with `vitray-widget` class

## License

MIT
