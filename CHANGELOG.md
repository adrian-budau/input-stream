# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/)

## [0.4.0] - 2019-06-02
### Added
- `scan_with_limit`

### Modified
- Error type is a simple enum

### Removed
- Removed failure dependency

## [0.3.0] - 2018-04-06
### Added
- Added clippy checks without using it as an optional dependency

### Modified
- Changed the error handling from [error-chain](https://crates.io/crates/error-chain)
to [failure](https://crates.io/crates/failure)

### Removed
- Removed clippy as an optional dependency

## [0.2.0] - 2017-02-04
### Added
- Now works on stable
- Documentation :smile:

### Modified
- Change clippy wildcard dependency to "0.0.\*"
- Moved benchmark to benches/ and using [rand](https://crates.io/crates/rand)
crate to generate the fixtures
- Result is exported
- Most lints became errors instead of warnings, code has been fixed to respect them

## Removed
- Removed fixtures

## [0.1.1] - 2016-12-25
### Added
- Clippy lints

## [0.1.0] - 2016-12-21
### Added

- Initial release of the input-stream crate, used for input parsing similar
to _istream_ in _C++_
