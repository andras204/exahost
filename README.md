# Exahost
---
Inspired by [EXAPUNKS](https://www.zachtronics.com/exapunks/), this project aims to bring distributed network programming with EXAs into the real world.

The project is currently still work in progress, and very much incomplete.

## Implementation status:
### Instruction interpreter
  - [x] Registers
    - [x] X
    - [x] T
	- [x] F
	- [ ] M
      - [x] local
      - [ ] global
	- [ ] Hardware
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
	- [x] test
      - [x] numbers
      - [x] keywords
      - [x] EOF
      - [x] MRD
    - [x] repl
    - [x] halt
    - [x] kill
    - [x] link
    - [x] host
    - [ ] mode
    - [x] void
      - [x] M
      - [x] F
    - [x] make
    - [x] grab
    - [x] file
    - [x] seek
    - [x] drop
    - [x] wipe
    - [x] note
    - [x] noop
    - [x] rand
  - [x] Macros
  - [ ] Extra Instructions // mainly for debug
    - [x] prnt

### Networking
  - [x] connect to other hosts
  - [ ] support for global M register
  - [ ] sync multiple hosts

### Host configuration
  - [ ] max connections
  - [x] max exas
  - [ ] bind address/port
  - [x] files
  - [ ] extra features
