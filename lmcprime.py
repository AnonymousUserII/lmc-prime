"""
This is a bad ripoff of the Little Man Computer, with extra steps
Instructions of use can be found in `README.md`
"""
from time import perf_counter_ns as perf_counter

COMMENT_CHAR: str = ';'
KEYWORDS: dict[str, int] = {"HLT": 0, "LDA": 1, "STA": 2, "ADD": 3, "SUB": 4,
                            "BRA": 5, "BRZ": 6, "BRP": 7, "DAT": 0}
EXT_KEYWORDS: dict[str, int] = {"INP": 8, "OUT": 9, "OTA": 10, "OTS": 11,
                                "OTB": 12, "OTC": 13}
MAX_12B: int = 2**12 - 1
MAX_13B: int = 2**13 - 1
MAX_15B: int = 2**15 - 1
MAX_16B: int = 2**16 - 1


def parse_line(raw_line: str) -> tuple | None:
    """
    Puts each term of line into a tuple, filtering comments
    :param raw_line: The line
    :return: The terms of the line in tuple, None if nothing valid
    """
    line_split = [term for term in raw_line.split(' ') if term != ""]
    
    try:
        line_split = line_split[:line_split.index(COMMENT_CHAR)]
    except ValueError:
        pass
    
    return line_split if len(line_split) else None


def parse_code(raw_assembly: str) -> tuple:
    """
    Parses the code for main process
    :param raw_assembly: Contents of LMCPrime file
    :return: Lines of parsed code
    """
    parsed_code: list = []
    
    for line in raw_assembly.split("\n"):
        parsed_line = parse_line(line)
        if parsed_line is not None:
            parsed_code.append(parsed_line)
    
    return tuple(parsed_code)


def check_syntax(code: tuple, print_error: bool = True) -> tuple[bool] | bool:
    """
    Checks if the parsed LMCPrime code is valid
    :param code: Parsed LMCPrime code
    :param print_error: Print the error it hits
    :return: (EXT, RET, LABELS) if true, False if invalid
    """
    def handle_err(message: str):
        if print_error:
            print(message)
        return False
    
    # Check if EXT, RET are present at beginning
    ext: bool = False
    ret: bool = False
    
    simp_bools = "0", "1", "FALSE", "TRUE"
    start_keywords = "EXT", "RET"
    start_lines = []
    for term in start_keywords:
        for b in simp_bools:
            start_lines.append(term + ' ' + b)
    for i, k in enumerate(start_keywords):
        if ' '.join(code[i]).upper() not in start_lines:
            return handle_err(f"Line {i} not in format: {k} (TRUE|FALSE)")
    
    # Set valid things for this code
    max_op_int = MAX_13B
    opcodes = KEYWORDS
    if ' '.join(code[0]).upper() in ("EXT TRUE", "EXT 1"):
        ext = True
        max_op_int = MAX_12B  # One more bit for instruction reduces size of operand 13->12
        opcodes = {**opcodes, **EXT_KEYWORDS}
    opcodes = opcodes.keys()  # Numeric opcodes not needed
    
    # Check opcodes and operands, acknowledge labels
    two_len_lines: list = []
    three_len_lines: list = []
    known_labels: dict[str, int] = {}  # Hold labels and their line number reference
    for i, line in enumerate(code[2:]):
        line_len: int = len(line)
        if line_len < 2 and ' '.join(line).upper() not in ("HLT", "DAT"):
            if ext is False or line[0] not in EXT_KEYWORDS:
                return handle_err(f"Line {i} has too few terms")
        if 3 < line_len:
            return handle_err(f"Line {i} has too many terms")
        
        if line_len == 2:  # Expected: OPCODE OPERAND or LABEL OPCODE (OPCODE can only be HLT or an EXT keyword)
            if line[0].upper() not in opcodes:  # Then must be LABEL OPCODE
                if line[1].upper() not in ("HLT", "DAT", *EXT_KEYWORDS):  # If OPCODE requires an operand
                    if line[1].upper() not in opcodes:
                        return handle_err(f"Line {i} has invalid opcode: {line[1]}")
                    return handle_err(f"Line {i} has too few terms")
                
                # Check if operand is a valid label
                if line[0].upper() in known_labels:  # If label already used
                    return handle_err(f"Line {i} uses label {line[0]} already used in line {known_labels[line[0]]}")
                if not line[0].isdigit():  # Don't treat number as label
                    known_labels[line[0].upper()] = i
                    continue
                
            two_len_lines.append((line, i))  # OPERAND may be a label, not all labels known
        
        elif line_len == 3:  # Expected: LABEL OPCODE OPERAND
            if line[0].upper() in opcodes:
                return handle_err(f"Line {i} has too many terms")
            else:
                if line[0].upper() in known_labels:  # If label already used
                    return handle_err(f"Line {i} uses label {line[0]} already used in line {known_labels[line[0]]}")
                if not line[0].isdigit():  # Don't treat number as label
                    known_labels[line[0].upper()] = i
            three_len_lines.append((line, i))
    
    for line in two_len_lines:
        # It is known that opcode is valid, but check if opcode is DAT
        if line[0][0].upper() == "DAT":
            if not 0 <= int(line[0][1]) <= MAX_16B:
                return handle_err(f"Line {line[1]}: data must be between 0 and {MAX_16B}")
        
        # Check if opcode is from extended, since they don't include an operand
        if line[0][0].upper() in EXT_KEYWORDS:
            continue
        
        # Check second term
        if line[0][1].isdigit():  # Numeral operand
            if not 0 <= int(line[1]) <= max_op_int:
                return handle_err(f"Line {line[1]}: operand must be between 0 and {max_op_int}")
        else:  # Potentially labelled operand
            if line[0][1].upper() not in known_labels:
                return handle_err(f"Line {line[1]} contains invalid operand {line[0][1]}")
    
    for line in three_len_lines:
        # LABEL is valid, check OPCODE and OPERAND
        if line[0][1].upper() not in opcodes:
            return handle_err(f"Line {line[1]} contains invalid opcode {line[0][1]}")
        
        # Check if opcode is DAT
        if line[0][1].upper() == "DAT":
            if not 0 <= int(line[0][2]) <= MAX_16B:
                return handle_err(f"Line {line[1]}: data must be between 0 and {MAX_16B}")
            continue
                    
        # Check if opcode is from extended, since they don't include an operand
        if line[0][0].upper() in EXT_KEYWORDS:
            continue
        
        # Check operand
        if line[0][2].isdigit():  # Numeral operand
            if not 0 <= int(line[0][2]) <= max_op_int:
                return handle_err(f"Line {line[1]}: operand must be between 0 and {max_op_int}")
        else:  # Potentially labelled operand
            if line[0][2].upper() not in known_labels:
                return handle_err(f"Line {line[1]} contains invalid operand {line[0][2]}")
    
        
    if ' '.join(code[1]).upper() in ("RET TRUE", "RET 1"):
        ret = True
    
    return ext, ret, known_labels


def set_mailboxes(code, labels, ext, keywords) -> tuple[int]:
    """
    Puts the assembly code into their respective mailboxes
    :return: The state of the mailboxes after loading in the code
    """
    mailboxes: list = [0 for _ in range(MAX_12B if ext else MAX_13B)] + [0]  # Account for exclusive range
    shift: int = 16 - (4 if ext else 3)
    
    for i, line in enumerate(code):
        if "HLT" in line:
            continue
        
        is_lk: bool = line[-1].upper() not in labels and len(line) == 2 and not line[-1].isdigit()  # Is LABEL KEYWORD
        
        # Add operand
        if line[-1].isdigit():
            mailboxes[i] += int(line[-1])
        elif line[0] not in EXT_KEYWORDS and not is_lk:
            mailboxes[i] += labels[line[-1].upper()]
        # Add opcode
        mailboxes[i] += keywords[line[-2 if (len(line) > 1 and not is_lk) else -1].upper()] << shift
    
    return mailboxes


def execute(mailboxes: list, ext: bool, ret: bool, printout: bool = False,
            code_length: int = None) -> int:
    """
    Runs the code from the first mailbox
    :return: Accumulator's final value
    """
    accumulator: int = 0
    program_counter: int = 0
    shift: int = 16 - (4 if ext else 3)
    
    opcodes = {**KEYWORDS, **EXT_KEYWORDS}
    
    while True:  # This is realistic (See: Halting Problem)
        if printout and code_length is not None:
            print(f"PC: {program_counter} |", 
                  f"ACC: {accumulator} ({accumulator:016b})")
            print_mailbox_range(mailboxes, ext, last=code_length - 1)
        instruction_hold = mailboxes[program_counter]
        opcode = instruction_hold >> shift
        operand = instruction_hold % 2 ** shift
        program_counter += 1
        
        if opcode == opcodes["HLT"]:
            break
        elif opcode == opcodes["LDA"]:
            accumulator = mailboxes[operand]
        elif opcode == opcodes["STA"]:
            mailboxes[operand] = accumulator
        elif opcode == opcodes["ADD"]:
            accumulator = (accumulator + mailboxes[operand]) % 2**16
        elif opcode == opcodes["SUB"]:
            accumulator = (accumulator - mailboxes[operand]) % 2**16
        elif opcode == opcodes["BRA"]:
            program_counter = operand
            continue
        elif opcode == opcodes["BRZ"]:
            if accumulator == 0:
                program_counter = operand
                continue
        elif opcode == opcodes["BRP"]:
            if 0 < accumulator < 2**15:
                program_counter = operand
                continue
        
        elif opcode == opcodes["INP"]:
            while True:
                try:
                    accumulator = int(input("Input: ")) % 2**16
                    break
                except ValueError:
                    print("Please enter a valid integer")
        elif opcode == opcodes["OUT"]:
            print(accumulator)
        elif opcode == opcodes["OTA"]:
            print(chr(accumulator % 2**8))
        elif opcode == opcodes["OTS"]:
            sign = accumulator >> 15
            print(accumulator % 2**16 - sign * MAX_16B)
        elif opcode == opcodes["OTB"]:
            print(f"{accumulator:016b}")
        elif opcode == opcodes["OTC"]:
            temp = f"{accumulator:016b}"
            print(temp.replace('0', ' '))
        else:
            print("Invalid opcode:", opcode)
            print("Make sure the program counter does not point to DAT\n" \
                  "Terminating program")
            return
    
    if ret:
        print(accumulator)
    return accumulator


def print_mailbox_range(mailboxes, ext, first: int = 0, last: int = None,
                        separate_ops: bool = True) -> None:
    """
    Prints mailbox contents of a range, all if no range given
    Contents displayed as binaries and representations
    """
    if last is None:
        last = len(mailboxes) - 1
    
    if separate_ops:
        keywords = {**KEYWORDS, **EXT_KEYWORDS} if ext else KEYWORDS
        reversed_opcodes = {v: k for k, v in reversed(keywords.items())}
        
        print("ADDRESS", "            " if ext else "             ",
              "OPCODE", "   " if ext else "    ", "OPERAND")
        for address in range(first, last + 1):
            opc_end: int = 4 if ext else 3
            content = f"{mailboxes[address]:016b}"
            
            try:
                ops: tuple = (f"{reversed_opcodes[int(content[:opc_end], 2)]} " \
                                f"({content[:opc_end]})",
                            f"{content[opc_end:]} " \
                                f"({f'{int(content[opc_end:], 2)}'})")
            except KeyError:  # Line is obviously a DAT
                ops: tuple = (f"DAT    {content[:opc_end]}" \
                              f"{content[opc_end:]} ({f'{int(content, 2)}'})",)
                pass
            print(f"{address:>4} ({address:0{f'{16 - opc_end}'}b}):",
                  f"{' '.join(ops)}")
    else:
        for address in range(first, last + 1):
            print(f"{address:>4}: {mailboxes[address]:016b}")
    print()
    return None


def main(file_path) -> None:
    with open(file_path, 'r') as file:
        file_contents = file.read()
    start = perf_counter()
    parsed_code = parse_code(file_contents)
    valid = check_syntax(parsed_code)
    
    if valid is False:
        print("Cannot parse LMCPrime code in", file_path)
        return None
    
    ext, ret, labels = valid
    kws = KEYWORDS
    if ext:
        kws = {**kws, **EXT_KEYWORDS}
    mailboxes: tuple[int] = set_mailboxes(parsed_code[2:], labels, ext, kws)
    
    execute(mailboxes, ext, ret, printout=False, code_length=len(parsed_code) - 1)
    print(f"Finished in {((perf_counter() - start)/1_000_000):.6f} ms")
    return 0


if __name__ == '__main__':
    from sys import argv
    if len(argv) > 1:
        code_to_run = argv[1]
    else:
        code_to_run = ""
    exit(main(code_to_run))
