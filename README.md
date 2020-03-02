# Bikeshed, a spec preprocessor - [Rust] version

A faithful rewrite of [bikeshed], a tool
used for generating many standards, in [Rust].

## Goal of this Project

The goal is to improve template performance for bigger projects. Ideally by
providing a drop in replacement for the [bikeshed] cli.

We're open to feasible alternatives to a full Rust rewrite. Moving performance
critical modules to Rust could be another option.

## Recommended Skill Set

Ideally you already wrote some Rust code and are somewhat experienced in reading
Python code. On top, having experience with any parser / template engine /
or text processing will help. Git knowledge will help but most of all being
able to adapt and handle a non-trivial code base may be key to succeed.

## Initial thoughts

_Assuming this will be (full) rewrite in Rust:_

As a first step we'll start setting up tests for this project. We copied tests
from [bikeshed] to [tests](../blob/master/tests) already. Next we'll create a
basic test harness. Ideally this is run by a simple `cargo test` as
[integration tests].

[bikeshed test implementation] will be used as an entry point to set this up.

Please do not open a PR against this repo to start a conversation. We try to
keep it fair and don't encourage you to do work beforehand.

[bikeshed]: https://github.com/tabatkins/bikeshed
[Rust]: https://www.rust-lang.org/
[integration tests]: https://doc.rust-lang.org/book/ch11-03-test-organization.html#integration-tests
[bikeshed test implementation]: https://github.com/tabatkins/bikeshed/blob/master/bikeshed/test.py#L44
[chat.mozilla.org]: https://chat.mozilla.org/#/room/#GSOC2020:mozilla.org