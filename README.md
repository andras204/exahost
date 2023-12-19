# Exahost
---
Inspired by [EXAPUNKS](https://www.zachtronics.com/exapunks/), this project aims to bring distributed network programming with EXAs into the real world.

The project is currently still work in progress, and very much incomplete.

Implementation status:
- [ ] Instruction interpreter
	- [ ] Registers
		- [x] X
		- [x] T
		- [ ] F
		- [ ] M
	- [ ] Base Instructions
		- [x] copy
		- [x] addi
		- [x] subi
		- [x] muli
		- [x] divi
		- [x] modi
		- [ ] swiz
		- [x] mark
		- [x] jump
		- [x] tjmp
		- [x] fjmp
		- [ ] test
			- [x] numbers
			- [ ] keywords
			- [ ] EOF
			- [ ] MRD
		- [x] repl
		- [x] halt
		- [x] kill
		- [ ] link
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
		- [ ] rand
	- [ ] Macros
	- [ ] Extra Instructions
		- [x] prnt
- [ ] Networking
	- [x] EXA serialization
	- [x] TCP connection to other Exahost instances
	- [x] EXA deserialization
        - [ ] link instruction
- [ ] Multithreading
