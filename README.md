# Exahost
---
Inspired by [EXAPUNKS](https://www.zachtronics.com/exapunks/), this project aims to bring distributed network programming with EXAs into the real world.

The project is currently still work in progress, and very much incomplete.

## Implementation status:
### Instruction interpreter
  - [ ] Registers
    - [x] X
    - [x] T
	- [ ] F
	- [ ] M
      - [x] local
      - [ ] global
  - [ ] Base Instructions
	- [x] copy
	- [x] addi
	- [x] subi
	- [x] muli
	- [x] divi
	- [x] modi
	- [x] swiz
	- [x] mark
	- [x] jump
	- [x] tjmp
	- [x] fjmp
	- [ ] test
      - [x] numbers
      - [x] keywords
      - [ ] EOF
      - [ ] MRD
    - [x] repl
    - [x] halt
    - [x] kill
    - [x] link
    - [ ] host
    - [ ] mode
    - [ ] void
      - [ ] M
      - [ ] F
    - [ ] make
    - [ ] grab
    - [ ] file
    - [ ] seek
    - [ ] drop
    - [ ] wipe
    - [x] note
    - [x] noop
    - [x] rand
  - [ ] Macros
  - [ ] Extra Instructions // mainly for debug
    - [x] prnt

### Networking
  - [x] connect to other hosts
  - [ ] support for global M register
  - [ ] sync multiple hosts

### Host configuration
  - [ ] max connections
  - [ ] max exas
  - [ ] bind address/port
  - [ ] files
  - [ ] extra features
