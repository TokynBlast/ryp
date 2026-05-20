# Tools
- MarkDown previewer
- Auto file reload on changes
- Complete marketplace integration

# Bugs
- Terminal ANSI escape sequences are incomplete
- Make terminal not cover cursor or bottom info
- Saving while in Git, saves the Git info to the actual file
- Search memory, not disk for project search
- When highlighting, then pressing arrow to go to the other side, cursor stays in same poosition
- Something like the "yes" command can consume all Ryp input

# QOL
- Implement larger terminal size (full screen?)
- Better documentation
- Resource limiting
- Explanation boxes for settings on hover
- Make settings more like the easier to manage VS Code settings
- Implement mouse support

# Internal
- Add minimum cells size
- Fetch a couple of hot items for start of marketplace, then when searching, only request the next 50 items on search, and not typing for one second
