# Tools
- MarkDown previewer
- Auto file reload on changes
- Complete marketplace integration

# Bugs
- Too many lines in the builtin terminal causes a crash
- Terminal ANSI escape sequences are incomplete
- Make terminal not cover cursor or bottom info
- Saving while in Git, saves the Git info to the actual file

# QOL
- Implement larger terminal size (full screen?)
- Better documentation
- Better bindings (or make them more logical)
- Resource limiting
- Explanation boxes for settings on hover

# Internal
- Add minimum cells size
- Implement function pointers for Lua tx detection
- Fetch a couple of hot items for start of marketplace, then when searching, only request the next 50 items on search, and not typing for one second
