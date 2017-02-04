#!/usr/bin/env python3
from os import environ
from subprocess import check_call

version = environ.get('TRAVIS_RUST_VERSION')
def run_command(command):
    print(' '.join(command))
    check_call(command)

extra_args = []
if version == 'nightly':
    extra_args = ['--features', 'clippy']

run_command(['cargo', 'build', '--verbose'] + extra_args)
run_command(['cargo', 'test', '--verbose'] + extra_args)

if version == 'nightly':
    run_command(['cargo', 'bench', '--verbose'] + extra_args)
