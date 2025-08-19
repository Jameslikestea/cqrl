---
layout: page
title: Change Log
permalink: /changelog
---
## v0.7.1 - 2025-08-19
### Documentation
- Add events and use-cases (#18)
- Update subjects to nats subjects

### Performance
- Build more binaries (#22)

### Chore
- Bump ring from 0.17.8 to 0.17.14 (#17)

## v0.7.0 - 2025-08-04
### Documentation
- Getting started (#14)

### Features
- Example clients and improved schema generation (#9)
- Url pagination and id filtering (#10)
- Authorization for requests (#12)
- Authorize nats requests (#13)

### Refactor
- Persistence chains

## v0.6.0 - 2025-07-16
### Features
- Add authentication for app_user type (#7)
- Rate limit on auth context (#8)

### Chore
- Update chains to accept arc

## v0.5.0 - 2025-07-08
### Documentation
- Add roadmap

### Features
- Add rate limiting for ip addresses

### Refactor
- Add authentication into chains

### Chore
- Format all code

## v0.4.0 - 2025-06-22
### Features
- Etag respect in HEAD and GET requests
- Add remote telemetry

### Chore
- Starting auth info

## v0.3.1 - 2025-05-30
### Bug Fixes
- Allow dead code
- Committer for changelog
- Update origin url
- Bump changelog version when outputting

### Documentation
- Add header to changelog

### Chore
- Output changelog to docs

## v0.3.0 - 2025-05-30
### Bug Fixes
- Assignment error in projection
- Unused variable assignment

### Features
- Server v2 authoring
- Update server version
- Add etags and body parsing

### Chore
- Issue templates (#3)
- Cargo fix

## v0.2.1 - 2025-05-04
### Bug Fixes
- Update filesystem import
- Use github app for release
- Add fs import
- Release with cURL
- Update references
- Fix release id

### Chore
- Update caching

## v0.2.0 - 2025-05-04
### Bug Fixes
- Remove release input
- Update changelog to only contain current release notes
- Use correct action
- No cross-compile
- Update octokit to github reference
- Define owner and repo constants
- Add release version to binary names

### Features
- Build artifacts for release

## v0.1.0 - 2025-05-04
### Bug Fixes
- Correct concurrency syntax
- Remove concurrency altogether
- Stop redeclaring context
- Bump version
- Version tagging
- Remove extra v prefix from tags

### Features
- Enable releases to be run

