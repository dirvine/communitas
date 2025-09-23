#!/bin/bash

echo "Checking Tauri app status..."
echo ""

# Check if the app is running
if pgrep -f "communitas-desktop" > /dev/null; then
    echo "✅ Tauri app process is running"
    echo "Process details:"
    ps aux | grep communitas-desktop | grep -v grep
else
    echo "❌ Tauri app process is NOT running"
fi

echo ""
echo "Checking web server..."
if curl -s http://localhost:1422 > /dev/null; then
    echo "✅ Web server is responding on port 1422"
else
    echo "❌ Web server is NOT responding on port 1422"
fi

echo ""
echo "Testing API endpoint..."
if curl -s http://localhost:1422/src/main.tsx | grep -q "React"; then
    echo "✅ React app files are being served"
else
    echo "❌ React app files are NOT being served correctly"
fi

echo ""
echo "To debug the Tauri window:"
echo "1. Right-click in the Tauri window and select 'Inspect Element' (if available)"
echo "2. Or press Cmd+Option+I to open DevTools"
echo "3. Check the Console tab for any errors"
echo ""
echo "If the window is truly blank, try:"
echo "1. Refresh the Tauri window with Cmd+R"
echo "2. Check if any content security policy is blocking resources"
