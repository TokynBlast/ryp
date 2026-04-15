# Pyr Lua
Pyr uses a modified version of Lua 5.5, to enable the development of plugins!

## What's gone?
Some things were removed from Lua, to avoid security issues, or things that would cause bugs.<br>
Everything

## What's new?
There's so many changes.
Not a lot of people will be used to these changes.

### Redirect of `print()`
Print no longer prints at the cursor.<br>
Instead, it prints to a debug console.

### Importance of nil
While the nil type still plays a part; nil is no longer a returned type.<br>
At least...<br>
Not by default.
Take the following code:

```
my_table = {
  obj1 = "Hello"
  obj2 = ", World!"
}
print(my_table.obj3)
```

This will error, instead of printing object 3.

### Type passing as Firt Class
Now, you can pass variable types as actual values!<br>
Here's a few examples:
```
some_func(type_passed)
  if type_passed == bool
    return true
  end
end
```
