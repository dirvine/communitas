#!/bin/bash

# Communitas MCP Setup Script
# This script installs and configures MCP servers for the Communitas project

YELLOW='\033[1;33m'
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${YELLOW}================================${NC}"
echo -e "${YELLOW}Communitas MCP Setup${NC}"
echo -e "${YELLOW}================================${NC}"
echo ""

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -d "communitas-desktop" ]; then
    echo -e "${RED}Error: This script must be run from the Communitas project root${NC}"
    echo "Please cd to /Users/davidirvine/Desktop/Devel/projects/communitas"
    exit 1
fi

echo -e "${GREEN}✓${NC} Running from Communitas project directory"
echo ""

# Check Node.js and npm
if ! command -v node &> /dev/null; then
    echo -e "${RED}✗ Node.js is not installed${NC}"
    echo "Please install Node.js first: brew install node"
    exit 1
fi

if ! command -v npm &> /dev/null; then
    echo -e "${RED}✗ npm is not installed${NC}"
    echo "Please install npm first: brew install node"
    exit 1
fi

echo -e "${GREEN}✓${NC} Node.js $(node --version) installed"
echo -e "${GREEN}✓${NC} npm $(npm --version) installed"
echo ""

# Function to install MCP server
install_mcp() {
    local package=$1
    local name=$2
    
    echo -e "${YELLOW}Installing $name...${NC}"
    
    # Try with npx first to ensure it's available
    if npx -y $package --version &>/dev/null || npx -y $package --help &>/dev/null; then
        echo -e "${GREEN}✓${NC} $name is available via npx"
    else
        # Try global installation
        npm install -g $package
        if [ $? -eq 0 ]; then
            echo -e "${GREEN}✓${NC} $name installed globally"
        else
            echo -e "${YELLOW}⚠${NC} $name may need manual installation"
        fi
    fi
}

# Install MCP servers
echo -e "${YELLOW}Installing MCP Servers...${NC}"
echo ""

install_mcp "tauri-mcp" "Tauri MCP"
install_mcp "chrome-devtools-mcp@latest" "Chrome DevTools MCP"
install_mcp "@modelcontextprotocol/server-filesystem" "Filesystem MCP"
install_mcp "@peakmojo/applescript-mcp" "AppleScript MCP"
install_mcp "@playwright/mcp@latest" "Playwright MCP"
install_mcp "@modelcontextprotocol/server-github" "GitHub MCP"
install_mcp "@modelcontextprotocol/server-memory" "Memory MCP"
install_mcp "@modelcontextprotocol/server-puppeteer" "Puppeteer MCP"

echo ""
echo -e "${YELLOW}Creating MCP memory directory...${NC}"
mkdir -p .mcp-memory
echo -e "${GREEN}✓${NC} Created .mcp-memory directory"

echo ""
echo -e "${YELLOW}Checking .gitignore...${NC}"
if ! grep -q ".mcp.json" .gitignore 2>/dev/null; then
    echo ".mcp.json" >> .gitignore
    echo -e "${GREEN}✓${NC} Added .mcp.json to .gitignore (contains GitHub token)"
else
    echo -e "${GREEN}✓${NC} .mcp.json already in .gitignore"
fi

if ! grep -q ".mcp-memory" .gitignore 2>/dev/null; then
    echo ".mcp-memory/" >> .gitignore
    echo -e "${GREEN}✓${NC} Added .mcp-memory to .gitignore"
else
    echo -e "${GREEN}✓${NC} .mcp-memory already in .gitignore"
fi

echo ""
echo -e "${YELLOW}Checking Tauri installation...${NC}"
if command -v cargo-tauri &> /dev/null; then
    echo -e "${GREEN}✓${NC} Tauri CLI installed"
else
    echo -e "${YELLOW}Installing Tauri CLI...${NC}"
    cargo install tauri-cli
fi

echo ""
echo -e "${GREEN}================================${NC}"
echo -e "${GREEN}MCP Setup Complete!${NC}"
echo -e "${GREEN}================================${NC}"
echo ""
echo "Next steps:"
echo "1. Restart Claude Desktop to load the new MCP configuration"
echo "2. Open this project in Claude Desktop"
echo "3. You should see new tools available:"
echo "   - Tauri app control"
echo "   - Filesystem operations"
echo "   - GitHub integration"
echo "   - Browser automation"
echo ""
echo "To test Tauri MCP:"
echo '  Ask: "Launch the Communitas desktop app"'
echo '  Ask: "Take a screenshot of the running app"'
echo ""
echo "Configuration file: .mcp.json"
echo "Documentation: MCP_SETUP.md"
echo ""

# Optional: Launch Claude Desktop
read -p "Would you like to restart Claude Desktop now? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "Restarting Claude Desktop..."
    osascript -e 'quit app "Claude"' 2>/dev/null
    sleep 2
    open -a "Claude" .
    echo -e "${GREEN}✓${NC} Claude Desktop restarted with Communitas project"
fi
