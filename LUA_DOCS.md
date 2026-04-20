# Pyr Lua
Pyr uses a modified version of Lua 5.5, to enable the development of plugins!

## What's gone?
Some things were removed from Lua, to avoid security issues, or things that would cause bugs.<br>
Everything

## What's new?
There's so many changes.
Not a lot of people will be used to these changes.

### Making a Plugin
There are two funcitons you should make.
`init()`, and `run()`.
The init function is the very first thing run.<br>
Depending on your priveleges, it will change what you're allowed to do.<br>
See [this](### Priveleges) for more info.

### Redirect of `print()`
Print no longer prints at the cursor.<br>
Instead, it prints to a debug console. The debug console can be opened via F12.

### Type passing as Firt Class
Now, you can pass variable types as actual values!<br>
Here's a few examples:
```
function some_func(type_passed)
  if type_passed == bool
    return true
  end
end
```

### Priveleges
The user is the ultimate priority fro Ryp.<br>
They are able to limit what you can, and cannot do.<br>
We provide a handy command to check your own priveleges, so you know what you can and cannot do before even starting.<br>