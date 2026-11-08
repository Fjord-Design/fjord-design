-- Headless runtime check; uses no external plugin, parser or language server.
local report = vim.json.decode(table.concat(vim.fn.readfile(vim.env.FJORD_NVIM_BUNDLE .. "/report.json"), "\n"))
for _, mode in ipairs({ "night", "day", "aurora" }) do
  vim.cmd.colorscheme("fjord-" .. mode)
  assert(vim.g.colors_name == "fjord-" .. mode)
  assert(vim.o.background == (mode == "day" and "light" or "dark"))
  for group, expected in pairs(report.modes[mode].highlights) do
    local actual = vim.api.nvim_get_hl(0, { name = group, link = false })
    for key, value in pairs(expected) do
      assert(actual[key] == (type(value) == "string" and tonumber(value:sub(2), 16) or value), mode .. ": " .. group .. "." .. key)
    end
  end
  for index = 0, 15 do assert(vim.g["terminal_color_" .. index]) end
  local namespace = vim.api.nvim_create_namespace("fjord-test")
  vim.api.nvim_buf_set_lines(0, 0, -1, false, { "local value = 42", "-- Readable comment" })
  vim.diagnostic.set(namespace, 0, { { lnum = 0, col = 0, message = "Fjord error", severity = vim.diagnostic.severity.ERROR } })
  assert(#vim.diagnostic.get(0) == 1)
  vim.diagnostic.reset(namespace, 0)
end
print("PASS Neovim native highlights, mode switch, ANSI palette and diagnostic API")
vim.cmd.qa({ bang = true })
