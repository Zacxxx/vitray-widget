#!/bin/bash
# Vitray Widget - Quick Install Script

set -e

REPO_URL="https://zacxxx.github.io/vitray-widget"
REPO_NAME="vitray-widget"

echo "🪟 Vitray Widget Installer"
echo "=========================="
echo ""

# Check if running as root
if [ "$EUID" -eq 0 ]; then 
   echo "⚠️  Please do not run as root. Run as normal user with sudo."
   exit 1
fi

# Add repository
echo "📦 Adding APT repository..."
echo "deb [trusted=yes] $REPO_URL stable main" | sudo tee /etc/apt/sources.list.d/$REPO_NAME.list > /dev/null

# Update package list
echo "🔄 Updating package list..."
sudo apt update

# Install
echo "⬇️  Installing vitray-widget..."
sudo apt install -y vitray-widget

echo ""
echo "✅ Installation complete!"
echo ""
echo "Launch with: vitray-widget"
echo "Or find 'Vitray Widget' in your application menu."
echo ""
echo "Add shortcuts with: vitray-widget --shortcut \"command\" \"name\""
echo "Right-click the widget to access settings."
