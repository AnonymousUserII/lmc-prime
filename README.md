# LMC Prime
This is a blatant ripoff of a Little Man Computer. See here: https://peterhigginson.co.uk/lmc/
Made in Python because

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
| Name | Description                                   |
|------|-----------------------------------------------|
| HLT  | Halt the program                              |
| LDA  | Load contents of address into accumulator     |
| STA  | Store contents of accumulator into address    |
| ADD  | Add contents of address to accumulator        |
| SUB  | Subtract contents of address from accumulator |
| JMP  | Jump to address                               |
| JMZ  | Jump to address if accumulator is zero        |
| JMN  | Jump to address if accumulator is non-zero    |

### Extended Instruction Set
| Name | Description                                               |
|------|-----------------------------------------------------------|
| INP  | Ask for user input into the accumulator                   |
| OUT  | Output contents of accumulator                            |
| OTA  | Output contents of accumulator as an ASCII character      |
| OTS  | Output contents of accumulator as a signed 16-bit integer |
| OTB  | Output contents of accumulator as a binary integer        |