for i, key in pairs({[0]="Left", [1]="Right"}) do
  hs.hotkey.bind("⌘⌥", key, function()
      local win = hs.window.focusedWindow()
      local f = win:frame()
      local screen = win:screen()
      local max = screen:frame()

      f.x = max.x + i * (max.w / 2)
      f.y = max.y
      f.w = max.w / 2
      f.h = max.h
      win:setFrame(f)
  end)
end

for i, key in pairs({[0]="Up", [1]="Down"}) do
  hs.hotkey.bind("⌘⌥", key, function()
      local win = hs.window.focusedWindow()
      local f = win:frame()
      local screen = win:screen()
      local max = screen:frame()

      f.x = max.x
      f.y = max.y + i * (max.h / 2)
      f.w = max.w
      f.h = max.h / 2
      win:setFrame(f)
  end)
end

hs.hotkey.bind("⌘⌥", "return", function()
    hs.window.focusedWindow():maximize()
end)

noResize = false
ensureInScreenBounds = true

hs.hotkey.bind("⌘⌥", "pageup", function()
    hs.window.focusedWindow():moveOneScreenNorth(noResize, ensureInScreenBounds)
end)

hs.hotkey.bind("⌘⌥", "pagedown", function()
    hs.window.focusedWindow():moveOneScreenSouth(noResize, ensureInScreenBounds)
end)

hs.hotkey.bind("⌘⌥", "home", function()
    hs.window.focusedWindow():moveOneScreenWest(noResize, ensureInScreenBounds)
end)

hs.hotkey.bind("⌘⌥", "end", function()
    hs.window.focusedWindow():moveOneScreenEast(noResize, ensureInScreenBounds)
end)

hs.hotkey.bind("⌘", "'", function()
    hs.application.launchOrFocus("Emacs")
end)

hs.hotkey.bind("⌘⌥", "'", function()
    hs.application.launchOrFocus("Dash")
end)

hs.hotkey.bind("⌘⌥", ";", function()
    hs.application.launchOrFocus("Firefox")
end)

hs.hotkey.bind("⌘", ";", function()
    hs.application.launchOrFocus("iTerm")
end)


-- hs.hotkey.bind("⌘⌥", "w", function()
--     local laptop = "Color LCD"
--     -- These have the same name, so we can't identify them only by name. See here: https://github.com/Hammerspoon/hammerspoon/issues/195
--     local dellVertical = hs.screen.allScreens()[3]
--     local dellHorizontal = hs.screen.allScreens()[1]
--     local top50 = hs.geometry.unitrect(0, 0, 1, 0.5)
--     local bottom50 = hs.geometry.unitrect(0, 0.5, 1, 0.5)
--     hs.layout.apply({
--         {"IntelliJ IDEA", nil, dellHorizontal, hs.layout.maximized, nil, nil},
--         {"Emacs", nil, dellHorizontal, hs.layout.maximized, nil, nil},
--         {"Firefox", nil, dellVertical, top50, nil, nil},
--         {"iTerm2", nil, dellVertical, bottom50, nil, nil},
--         {"Slack", nil, laptop, hs.layout.maximized, nil, nil},
--         {"Cathode", nil, laptop, hs.layout.maximized, nil, nil},
--     })
-- end)
