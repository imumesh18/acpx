default:
  @just --list

fmt:
  devenv tasks run fmt:write

fmt-check:
  devenv tasks run fmt:check

lint:
  devenv tasks run lint:check

lint-fix:
  devenv tasks run lint:fix

test:
  devenv tasks run test:check

example name:
  devenv tasks run --input name={{ quote(name) }} example:check

audit:
  devenv tasks run audit:check

build:
  devenv tasks run build:debug

doc:
  devenv tasks run doc:build

quality:
  devenv tasks run quality:check

quality-fix:
  devenv tasks run quality:fix

changelog:
  devenv tasks run changelog:update

release-notes version='':
  devenv tasks run --show-output --input version={{ quote(version) }} release:notes

next-version:
  @devenv tasks run --show-output version:next

ref-clone url name='':
  devenv tasks run --input url={{ quote(url) }} --input name={{ quote(name) }} ref:clone

ref-copy source name:
  devenv tasks run --input source={{ quote(source) }} --input name={{ quote(name) }} ref:copy

registry-sync:
  devenv tasks run registry:sync

publish-dry-run:
  devenv tasks run --input dry_run=true publish:run

release version='':
  devenv tasks run --input version={{ quote(version) }} release:prepare
