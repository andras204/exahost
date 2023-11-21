# Exahost
---
Inspired by [EXAPUNKS](https://www.zachtronics.com/exapunks/), this project aims to bring distributed network programming with EXAs into the real world.

The project is currently still work in progress, and very much incomplete.

Implementation status:
- [ ] Exahost
	- [ ] Instruction interpreter
		- [ ] Registers
			- [x] X
			- [x] T
			- [ ] F
			- [ ] M
		- [ ] Instructions
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
			- [x] prnt
			- [ ] repl
			- [ ] halt
			- [ ] kill
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
			- [ ] note
			- [ ] noop
			- [ ] rand
	- [ ] Networking
		- [ ] EXA serialization
		- [ ] TCP connection to other Exahost instances
		- [ ] EXA deserialization
	- [ ] Multithreading