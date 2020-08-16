import vim
from omnipytent import *
from omnipytent.ext.idan import *


@task
def compile(ctx):
    cargo['build', '-q'] & ERUN.bang


@task
def run(ctx):
    cargo['run', '-q'].with_env(RUST_BACKTRACE='1', RUST_LOG='chapter_tracker=debug') & BANG


@task
def test(ctx):
    cargo['test', '-q', '--package=chapter-tracker', '--', '--nocapture', '--quiet', '--test'].with_env(RUST_BACKTRACE='1') & ERUN


@task
def go(ctx, example=cargo_example):
    cargo['run', '-q', '--example', example].with_env(RUST_BACKTRACE='1', RUST_LOG='chapter_tracker=debug') & BANG
