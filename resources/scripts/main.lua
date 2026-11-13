--[[
  Набросок скрипта. Буду постепенно двигаться к "протодвижку",
  который будет работать через Lua-скрипты и быть чем-то наподобие Blitz3D/BlitzMax
]]--

local cranberry = require('cranberry')

-- Базовая иницилизация
local app = cranberry.Application.new({
  fullscreen = true,
  width = 1280,
  height = 1024,
  renderer = cranberry.Renderer.OpenGl, -- Вдруг я когда-нибудь "дорасту" до Vulkan?
});

local camera = cranberry.Camera.new(app, cranberry.CameraType.Perspective)

local box = cranberry.Mesh.box(app, 0.5, 1.5)
box:move(0.0, 0.0, -1.0)
box:rotate(45.0, 30.0, 60.0)

app:on_next_frame(function (delta_factor)
  cranberry.debug_log('Delta factor:' .. delta_factor)
  if app:is_key_pressed(cranberry.KeyboardKey.Escape) then
    app:stop();
  end
end)

app:start();
