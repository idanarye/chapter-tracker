local moonicipal = require'moonicipal'
local T = moonicipal.tasks_file()

function T:check()
    vim.cmd'Erun! cargo check -q --examples'
end

function T:build()
    vim.cmd[[
    botright new
    terminal cargo build --examples
    startinsert
    ]]
end

function T:run()
    vim.cmd[[
    botright new
    terminal RUST_BACKTRACE=1 RUST_LOG=chapter_tracker=debug cargo run -- --linksdir episodes-links
    startinsert
    ]]
end
