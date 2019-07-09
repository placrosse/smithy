# Version 0.0.4 (2019-06-26)

## Features and breaking changes

* Renamed `smd_no_move!` to `smd_borrowed!`
* Cache calls to `smd!` and `smd_borrowed!` to improve compile times
* Rename the "debug-logs" feature to "browser-logs"
* Additional events documentation

## Bugs

* Fix a bug that caused smithy to panic when updating some interpolated
  variables (e.g. if `count` was updated in `<div>{ count }</div>`
* Fix a bug allowing certain event related features to be enabled
  (including all-features)

# Version 0.0.3 (2019-04-29)

* Add the `smd_borrowed!` macro
* Add documentation
* fix smd!() not compiling
* fix unused import compiler warnings
* add features for global events: `before-unload-events`, `hash-change-events`, `pop-state-events`, and `promise-rejection-events`
* add post-rendering tests