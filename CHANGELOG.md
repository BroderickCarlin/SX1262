# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.1](https://github.com/BroderickCarlin/SX1262/compare/v0.3.0...v0.3.1) - 2025-10-24

### Other

- remove unneeded readme section

## [0.3.0](https://github.com/BroderickCarlin/SX1262/compare/v0.2.2...v0.3.0) - 2025-10-24

### Other

- Add missing status byte to some commands
- Update regiface and use new Zeros type

## [0.2.2](https://github.com/BroderickCarlin/SX1262/compare/v0.2.1...v0.2.2) - 2025-10-22

### Fixed

- RX and TX timeout should be 24 bits, not 32 bits

### Other

- Fix CI failure
- Merge pull request #9 from BroderickCarlin/dependabot/github_actions/taskmedia/action-conventional-commits-1.1.20
- Merge pull request #10 from BroderickCarlin/dependabot/cargo/defmt-1.0
- Merge pull request #12 from MarSik/main

## [0.2.1](https://github.com/BroderickCarlin/SX1262/compare/v0.2.0...v0.2.1) - 2024-12-18

### Added

- Add async variations of various methods, and documentation

## [0.2.0](https://github.com/BroderickCarlin/SX1262/compare/v0.1.2...v0.2.0) - 2024-12-18

### Added

- Add a Device type to simplify interfacing with a device

### Fixed

- Fix some issues in the command/register types

## [0.1.2](https://github.com/BroderickCarlin/SX1262/compare/v0.1.1...v0.1.2) - 2024-12-13

### Fixed

- Pin to regiface 0.2.2 to fix no_std compat

## [0.1.1](https://github.com/BroderickCarlin/SX1262/compare/v0.1.0...v0.1.1) - 2024-12-13

### Fixed

- Remove std dependencies and mark crate as no_std

## [0.1.0](https://github.com/BroderickCarlin/SX1262/releases/tag/v0.1.0) - 2024-12-09

### Added

- initial commit

### Other

- update README with comprehensive project documentation
- Change crate name to avoid conflict
- Initial commit
