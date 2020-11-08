# aocleaderboard

[Advent of Code](https://adventofcode.com) private leaderboards are currently
limited to 200 users. This simple web app can merge users from multiple
leaderboards, recalculate their scores based on the total number of members, and
present them in a single page with the same look and feel of the original
Advent of Code website. Leaderboards are fetched in JSON format from the
adventofcode.com API URL.

## Requirements

This app uses [Rocket](https://rocket.rs) web framework and, therefore, requires
a _nightly_ version of Rust.

## Setup

TODO

## Features

- To avoid overloading the Advent of Code website, leaderboards are cached in
   memory and fetched again after a configurable time limit (by default, 15
   minutes).
- Leaderboards can be ordered by local score (based on the time each star was
  acquired) or by number of stars. Tie breaks are broken by the time the most
  recent star was acquired.

## Support Advent of Code

Advent of Code is a free online Advent calendar of small programming puzzles
created by [Eric Wastl](http://was.tl/) and maintained by volunteers. Please
consider [supporting their work](https://adventofcode.com/support).


