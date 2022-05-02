import vim
from omnipytent import *
from omnipytent.ext.idan import *


@task
def build(ctx):
    cargo['build', '--examples'] & TERMINAL_PANEL

@task
def check(ctx):
    cargo['check', '-q', '--examples'] & ERUN.bang


@task
def run(ctx):
    cargo['run', '--', '--linksdir', 'episodes-links'].with_env(RUST_BACKTRACE='1', RUST_LOG='chapter_tracker=debug') & TERMINAL_PANEL
    # cargo['run'].with_env(RUST_BACKTRACE='1', RUST_LOG='chapter_tracker=debug') & TERMINAL_PANEL
    # cargo['run'].with_env(RUST_BACKTRACE='1', RUST_LOG='chapter_tracker=debug') & TERMINAL_PANEL


@task
def test(ctx):
    cargo['test', '-q', '--package=chapter-tracker', '--', '--nocapture', '--quiet', '--test'].with_env(RUST_BACKTRACE='1') & ERUN


@task
def go(ctx, example=cargo_example):
    cargo['run', '-q', '--example', example].with_env(RUST_BACKTRACE='1', RUST_LOG='chapter_tracker=debug') & TERMINAL_PANEL.size(40)


@task
def add_migration(ctx, name):
    local['sqlx']['migrate', 'add', name].with_env(DATABASE_URL='sqlite:chapter_tracker.db3')()


@task(alias='init')
def reset_db(ctx):
    local['cp']['chapter_tracker.db3.old']['chapter_tracker.db3'] & BANG


@task
def install(ctx):
    local['./install-script.sh'] & TERMINAL_PANEL
