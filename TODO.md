# Tools
- MarkDown previewer
- Ctrl+Arrow skips text
- Auto file reload on changes
- Lua plugins

# Bugs
- Too many lines in the builtin terminal causes a crash
- Terminal ANSI escape sequences are incomplete
- delete key does nothing
- Make terminal not cover cursor or bottom info
- Saving while in Git, saves the Git info to the actual file

# QOL
- Implement larger terminal size (full screen?)
- Better documentation
- Better bindings (or make them more logical)
- Update only changed cells
- Resource limiting
- Copy, cut, and paste (via typical bindings)

# Internal
- Add minimum cells size
- Test caches as arc-swap
- Test rayon on constant cache rebuild
