# Contributing to translatable

In translatable we welcome any contribution from anyone, bug reports, pull requests, and feedback.
This document serves as guidance if you are thinking of submitting any of the above.

## Submitting bug reports and feature requests

To submit a bug report or feature request you can open an issue in this repository `FlakySL/translatable`.

When reporting a bug or asking for help, please include enough details so that the people helping you
can reproduce the behavior you are seeking. For some tips on how to approach this, read about how to
produce a (Minimal, Complete, and Verifiable example)[https://stackoverflow.com/help/minimal-reproducible-example].

When making a feature request, please make it clear what problem you intend to solve with the feature,
any ideas for how translatable could support solving that problem, any possible alternatives, and any
disadvantages.

Before submitting anything please, check that another issue with your same problem/request does not
already exist, if you want to extend on a problem or have an actual conversation about it, you can
use our discord channel [at Flaky](https://discord.gg/AJWFyps23a).

It is recommended that you use the issue templates provided in this repository.

## Running tests and compiling the project

This project uses [`make`](https://www.gnu.org/software/make/) for everything you may want to run.

- To run tests you can use the `make test` command, that runs both integration and unit tests.
- To compile the project alone you may use `make compile`, that simply compiles the project in each
target directory.

## Code of conduct

In translatable and community we abide by the [Rust code of conduct](https://www.rust-lang.org/policies/code-of-conduct).
For escalation or moderation issues please contact Esteve ([esteve@memw.es](esteve@memw.es)) instead of the Rust moderation team.
