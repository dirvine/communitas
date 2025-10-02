// Example AppleScript commands for testing your Tauri app with the AppleScript MCP

// 1. Launch your Tauri app
tell application "Communitas" to activate

// 2. Get window information
tell application "Communitas"
    set windowBounds to bounds of window 1
    set windowName to name of window 1
    return {windowName, windowBounds}
end tell

// 3. Click a button in your app
tell application "System Events" to tell process "Communitas"
    click button "Connect" of window 1
end tell

// 4. Type text into a field
tell application "System Events" to tell process "Communitas"
    set value of text field 1 of window 1 to "test@example.com"
end tell

// 5. Send keyboard shortcuts
tell application "System Events" to tell process "Communitas"
    keystroke "n" using command down -- Cmd+N
end tell

// 6. Take a screenshot of your app window
do shell script "screencapture -l$(osascript -e 'tell app \"Communitas\" to id of window 1') ~/Desktop/communitas-screenshot.png"

// 7. Get all UI elements (useful for discovering what to automate)
tell application "System Events" to tell process "Communitas"
    entire contents of window 1
end tell

// 8. Check if app is running
tell application "System Events"
    set appList to name of every process
    if "Communitas" is in appList then
        return "App is running"
    else
        return "App is not running"
    end if
end tell

// 9. Resize and position window
tell application "Communitas"
    set bounds of window 1 to {100, 100, 1200, 800}
end tell

// 10. Close and reopen app
tell application "Communitas" to quit
delay 2
tell application "Communitas" to activate
