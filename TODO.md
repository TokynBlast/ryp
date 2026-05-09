# Tools
- MarkDown previewer
- Auto file reload on changes
- Lua plugins

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
- Implement cursor versions of all setters and getters for Lua
  - editor.str.get.cursor()
  - editor.str.set.cursor(string)
  - editor.str.get({x, y})
  - editor.str.set({x, y}, string)

# Internal
- Add minimum cells size
- Implement function pointers for Lua tx detection
