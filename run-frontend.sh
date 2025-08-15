#!/bin/bash

echo "Starting Communitas Frontend (Development Mode)"
echo "============================================="
echo ""
echo "This will run the frontend without the Tauri backend."
echo "Some features that require native APIs will not work."
echo ""
echo "Starting Vite dev server..."
echo "Access the app at: http://localhost:1420"
echo ""

# Kill any existing Vite processes on port 1420
lsof -ti:1420 | xargs kill -9 2>/dev/null

# Run the dev server
npm run dev