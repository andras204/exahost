# Exahost
---
Inspired by [EXAPUNKS](https://www.zachtronics.com/exapunks/), this project aims to answer the question of "Is it possible to transfer a program across a network while it is still running?".

The answer is yes! (According to POC tests)

this project is still very much WIP, currently it has no user interface (GUI, TUI, or CLI).


# How
---
The project consist of 3 major components:
  - compiler
  - VM
  - server

The compiler compiles EXA Language (almost the same as in the game) into bytecode.

The VM executes the bytecode

The server handles connections to other `exahost` instances, and is responsible for sending/recieving `Exa`s
