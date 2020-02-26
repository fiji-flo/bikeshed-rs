# Bikeshed, a spec preprocessor - [Rust] version

A faithful rewrite of [bikeshed], a tool
used for generating many standards, in [Rust].

## Goal of this Project

The goal is to improve template performance for bigger projects. Ideally by
providing a drop in replacement for the [bikeshed] cli.

We're open to feasible alternatives to a full Rust rewrite. Moving performance
critical modules to Rust could be another option.

## First Steps

As a first step please start setting up tests for this project. We copied tests
from [bikeshed] to [tests](../blob/master/tests). Please start by creating a
basic test harness. Ideally this is run by a simple `cargo test` as
[integration tests].

Look at the [bikeshed test implementation] for a good entry point.

Feel free to open a PR against this repo to start a conversation.

[bikeshed]: https://github.com/tabatkins/bikeshed
[Rust]: https://www.rust-lang.org/
[integration tests]: https://doc.rust-lang.org/book/ch11-03-test-organization.html#integration-tests
[bikeshed test implementation]: https://github.com/tabatkins/bikeshed/blob/master/bikeshed/test.py#L44