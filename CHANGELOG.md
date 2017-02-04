# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/)

## [Unreleased]
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
