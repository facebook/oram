## oram ![Build Status](https://github.com/facebook/oram/workflows/CI/badge.svg)

This library implements an Oblivious RAM (ORAM) for secure enclave applications.

This crate assumes that ORAM clients are running inside a secure enclave architecture that provides memory encryption.
It does not perform encryption-on-write and thus is **not** secure without memory encryption.

⚠️ **Warning**: This implementation has not been audited and is not ready for use in a real system. Use at your own risk!

Documentation
-------------

TODO link to docs.rs page after the crate is published.

Installation
------------

TODO after crate is published

### Minimum Supported Rust Version

TODO

Resources
---------

- [Original Path ORAM paper](https://eprint.iacr.org/2013/280.pdf), which introduced the standard "vanilla" variant of Path ORAM on which this library is based.
- [Path ORAM retrospective paper](http://elaineshi.com/docs/pathoram-retro.pdf), containing a high-level overview of developments related to Path ORAM.
- [Oblix paper](https://people.eecs.berkeley.edu/~raluca/oblix.pdf), which describes the oblivious stash data structure this library implements. 

Contributors
------------

The authors of this code are Spencer Peters ([@spencerpeters](https://github.com/spencerpeters)) and Kevin Lewi
([@kevinlewi](https://github.com/kevinlewi)).
To learn more about contributing to this project, [see this document](./CONTRIBUTING.md).

Code Organization
--------------------
Within `src/`:
- `lib.rs` defines the `Oram` trait and public API.
- `path_oram.rs` defines the main ORAM implementation.
- `position_map.rs` and `stash.rs` define the oblivious position map and stash respectively.
- `bucket.rs` defines low-level block and bucket structs.
- `linear_time_oram.rs` contains a trivial linear-time ORAM implementation used as a base case.
- `database.rs` defines a simple RAM abstraction (to be removed).
- `utils.rs` contains utilities related to oblivious sorting and tree index calculations.
- `test_utils` contains code shared between tests.

License
-------

This project is dual-licensed under either the [MIT license](https://github.com/facebook/oram/main/LICENSE-MIT)
or the [Apache License, Version 2.0](https://github.com/facebook/oram/blob/main/LICENSE-APACHE).
You may select, at your option, one of the above-listed licenses.
