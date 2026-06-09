# ThothTerm Demo — VHS tape
# Generate: ~/go/bin/vhs demo.tape

Output demo.gif

Set Theme "Catppuccin Mocha"
Set FontSize 14
Set Width 1100
Set Height 620
Set Shell "bash"
Set Padding 12
Set PlaybackSpeed 0.85

Sleep 300ms

# Title
Type "printf '\\033[1;33m𓆣 ThothTerm — GPU terminal for the next 10 years\\033[0m\\n'"
Enter
Sleep 600ms

# Version
Type "printf '\\033[1;32mv0.1.0\\033[0m  (Rust + wgpu + wasmtime)\\n'"
Enter
Sleep 700ms

# Spacer
Type "echo"
Enter

# Plugin install
Type "# Install a plugin"
Enter
Sleep 200ms
Type "thothterm plugin install ./custom_crates/example-plugins/thothterm-plugin-hello"
Enter
Sleep 800ms

# Plugin list
Type "thothterm plugin list"
Enter
Sleep 800ms

# Feature table
Type "echo"
Enter
Type "printf '\\033[1;36m%-26s %s\\033[0m\\n' 'Shortcut' 'Feature'"
Enter
Type "printf '%-26s %s\\n' 'Ctrl+,'        '→ GUI settings panel'"
Enter
Type "printf '%-26s %s\\n' 'Ctrl+Shift+P'  '→ WASM plugin manager'"
Enter
Type "printf '%-26s %s\\n' 'Ctrl+Shift+A'  '→ AI chat (Ollama / OpenAI)'"
Enter
Type "printf '%-26s %s\\n' 'Ctrl+Shift+W'  '→ Web3 panel + ENS + gas ⛽'"
Enter
Sleep 1.2s

# Error AI demo
Type "echo"
Enter
Type "# AI explains errors automatically (OSC 133 shell integration)"
Enter
Sleep 300ms
Type "ls /nonexistent 2>&1; printf '\\033[31mExit: %d\\033[0m\\n' $?"
Enter
Sleep 800ms

# Portable mode
Type "echo"
Enter
Type "# Portable mode — put .portable next to the exe, all data stays local"
Enter
Sleep 300ms
Type "ls -la ./ThothTerm-portable/ 2>/dev/null || printf '  config/  plugins/  data/  .portable\\n'"
Enter
Sleep 800ms

# Outro
Type "echo"
Enter
Type "printf '\\033[1;33m𓆣  github.com/Lord1Egypt/ThothTerm\\033[0m\\n'"
Enter
Sleep 1.8s
