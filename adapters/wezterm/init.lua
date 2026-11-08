-- MIT; copyright 2026 Fjord contributors. Apply colours only to the caller's config.
local wezterm = require 'wezterm'
local fjord = {}
function fjord.apply(config, mode, directory)
  directory = directory or (wezterm.config_dir .. "/fjord/")
  if type(directory) ~= "string" or directory == "" then error("Fjord needs a scheme directory") end
  if directory:sub(-1) ~= "/" then directory = directory .. "/" end
  if mode ~= 'night' and mode ~= 'day' and mode ~= 'aurora' then
    error('Fjord mode must be night, day or aurora')
  end
  local scheme = wezterm.color.load_scheme(directory .. mode .. '.toml')
  local merged = {}
  for key, value in pairs(config.colors or {}) do merged[key] = value end
  for key, value in pairs(scheme) do merged[key] = value end
  config.colors = merged
  return config
end
return fjord
