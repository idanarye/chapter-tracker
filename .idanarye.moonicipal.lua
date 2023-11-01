local moonicipal = require'moonicipal'
local T = moonicipal.tasks_file()

moonicipal.include(require'idan.project.rust' {
    crate_name = 'chapter_tracker',
    cli_args_for_targets = {
        ['chapter-tracker'] = {
            {'--linksdir', 'episodes-links'},
        }
    },
})

local blunder = require'blunder'
local channelot = require'channelot'

function T:add_migration()
    local migration_name = moonicipal.input { prompt = 'Migration Name> ' } or moonicipal.abort()
    if migration_name == '' then
        return
    end
    blunder.create_window_for_terminal()
    channelot.terminal_job({
        DATABASE_URL='sqlite:chapter_tracker.db3',
    }, {'sqlx', 'migrate', 'add', migration_name})
end

function T:copy_the_real_db()
    blunder.create_window_for_terminal()
    local t = channelot.terminal()
    t:job{'cp', '--verbose', '/media/d/ChapterTracker/chapter_tracker.db3', 'chapter_tracker.db3.old'}:wait()
    t:job{'cp', '--verbose', 'chapter_tracker.db3.old', 'chapter_tracker.db3'}:wait()
    t:prompt_exit()
end

function T:reset_db()
    blunder.create_window_for_terminal()
    vim.fn.termopen{'cp', '--verbose', 'chapter_tracker.db3.old', 'chapter_tracker.db3'}
end
