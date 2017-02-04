#!/usr/bin/env python3
from os import system, environ

version = environ.get('TRAVIS_RUST_VERSION')
def run_command(command):
    print(command)
    system(command)

extra_args = ''
if version == 'nightly':
    extra_args = '--features clippy'

run_command('cargo build --verbose ' + extra_args)
run_command('cargo test --verbose ' + extra_args)

if version == 'nightly':
    run_command('cargo bench --verbose ' + extra_args)
