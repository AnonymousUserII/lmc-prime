# LMC Prime
This is a blatant ripoff of a Little Man Computer. See here: https://peterhigginson.co.uk/lmc/

Made in Python because

`/lmc-prime` contains an extension for Visual Studio Code if syntax highlighting is desired.

## Differences
This project was designed as a 16-bit system instead of a 3-digit.
Furthermore, the number of mailboxes has drastically increased from 100 to as many as can be pointed to by the operand.
*Naturally*, all arithmetic is performed using unsigned 16-bit integers.

The assembly code has two prefixing lines, EXT and RET.
* EXT determines whether the code will use the extended instruction set or not
  * These currently include console input and output
  * This will take up an extra bit for the instructions, leaving only 12 bits for the operands
* RET determines whether the code will print out the final value of the accumulator after halting

## Instruction Set
| Name  | Opcode    | Description                                   |
|-------|-----------|-----------------------------------------------|
| `HLT` | `0 (000)` | Halt the program                              |
| `LDA` | `1 (001)` | Load contents of address into accumulator     |
| `STA` | `2 (010)` | Store contents of accumulator into address    |
| `ADD` | `3 (011)` | Add contents of address to accumulator        |
| `SUB` | `4 (100)` | Subtract contents of address from accumulator |
| `JMP` | `5 (101)` | Jump to address                               |
| `JMZ` | `6 (110)` | Jump to address if accumulator is zero        |
| `JMN` | `7 (111)` | Jump to address if accumulator is negative as 16-bit signed integer    |

### Extended Instruction Set
| Name  | Opcode      | Description                                               |
|-------|-------------|-----------------------------------------------------------|
| `INP` | ` 8 (1000)` | Ask for user input into the accumulator                   |
| `OUT` | ` 9 (1001)` | Output contents of accumulator                            |
| `OTA` | `10 (1010)` | Output contents of accumulator as an ASCII character      |
| `OTS` | `11 (1011)` | Output contents of accumulator as a signed 16-bit integer |
| `OTB` | `12 (1100)` | Output contents of accumulator as a binary integer        |
