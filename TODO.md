# Tools
- MarkDown previewer
- Auto file reload on changes
- Complete marketplace integration

# Bugs
- Terminal ANSI escape sequences are incomplete
- Make terminal not cover cursor or bottom info
- Saving while in Git, saves the Git info to the actual file
- Search memory, not disk for project search
- When highlighting, then pressing arrow to go to the other side, cursor stays in same position
- Something like the "yes" command can consume all Ryp input in GUI while in a terminal
- Local find isn't UTF-8 safe

# QOL
- Resource limiting
- Explanation boxes for settings on hover
- Make settings more like the easier to manage VS Code settings
- Implement mouse support where supported
- Implement an image decoder that has no file access after the image is grabbed (prevents overflow buffer execution and other security risks with images that contain malicous code [this is possible, and does happen])

# Internal
- Fetch a couple of hot items for start of marketplace, then when searching, only request the next 50 items on search, and not typing for one second
