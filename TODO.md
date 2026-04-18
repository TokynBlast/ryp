# Tools
- MarkDown preview viewer
- Ctrl+Arrow fast move
- Auto file reload on changes
- Lua plugins

# Bugs
- Too many lines in the builtin terminal causes a crash
- vt100 support is incomplete
- del does nothing
- Make terminal not cover cursor or bottom info
- Saving while in Git, saves the Git info to the actual file

# QOL
- Implement Luna terminal
- Implement larger terminal (full terminal?)
- Better documentation
- Better bindings (or make them more logical)
- Update only changed cells
- Resource limiting
- Copy, cut, and paste (via typical bindings)

# Internal
- Add minimum cells size
- Test caches as arc-swap
- Implement file names and other likely to be small strings via CompactStr
