# Things to add

When there are several options to try, they're ordered by preference from cursory research.

## Check out iTerm2's option "Use built-in Powerline glyphs"

## Create bin/ directory

## japi-cc

## Set default browser to Firefox

## macOS User Defaults

Use our seteuid trick

Then write a C or Objective-C function that uses [`CFPreferencesSetMultiple`](https://developer.apple.com/documentation/corefoundation/1515513-cfpreferencessetmultiple?language=objc)

Call [`CFPreferencesSynchronize`](https://developer.apple.com/documentation/corefoundation/1515504-cfpreferencessynchronize?language=objc) afterward.

This way we don't have to write any complicated FFI. We'll just call a big function from Rust.

Other option is to write an FFI for 4 types: bool, int, float, and string. Doesn't really seem like we were settings and arrays.

Use [`CFPreferencesSetAppValue`](https://developer.apple.com/documentation/corefoundation/1515528-cfpreferencessetappvalue?language=objc) for this.

Example:

```rust
Application::new("com.apple.menuextra.clock")
    .string("DateFormat", "EEE MMM d  H:mm")
    .bool("FlashDateSeparators", false)
    .sync()?;
```

Call `CFPreferencesSynchronize` on `sync`. We can't do it on drop because drop cannot fail.

Use vROps Deploy's caffeine.c as an example of creating a `CFNumber`.

## Zsh plugin manager options

1. https://github.com/zdharma/zinit
1. https://github.com/zplug/zplug
1. https://github.com/rossmacarthur/sheldon
1. http://getantibody.github.io/

https://github.com/vintersnow/zsh_plugin_manager_speed
https://www.reddit.com/r/zsh/comments/ak0vgi/a_comparison_of_all_the_zsh_plugin_mangers_i_used/
https://jdhao.github.io/2019/10/08/zsh_plugin_managers_compare/

## Prompts

1. https://github.com/eendroroy/alien
1. https://github.com/reujab/silver
