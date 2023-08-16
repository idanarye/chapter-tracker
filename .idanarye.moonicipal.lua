local moonicipal = require'moonicipal'
local T = moonicipal.tasks_file()

T = require'idan.project.rust'(T, {
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
