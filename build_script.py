#!/usr/bin/env python
from __future__ import print_function
from os import environ
from subprocess import check_call, check_output, STDOUT

version = environ.get('TRAVIS_RUST_VERSION')
def run_command(command, fail_on_output=False):
    print(' '.join(command))
    if not fail_on_output:
        check_call(command)
    else:
        output = None
        try:
            output = check_output(command, stderr=STDOUT)
            if len(output) > 0:
                raise Exception("Expected no output")
        finally:
            print(output.decode('utf-8'))

extra_args = []
if version == 'nightly':
    extra_args = ['--features', 'clippy']

run_command(['cargo', 'build', '--verbose'] + extra_args)
run_command(['cargo', 'test', '--verbose'] + extra_args)
run_command(['cargo', 'doc', '--verbose'])
run_command(['cargo', 'deadlinks'], fail_on_output=True)

if version == 'nightly':
    run_command(['cargo', 'bench', '--verbose'] + extra_args)
