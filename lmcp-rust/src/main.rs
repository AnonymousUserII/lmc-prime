/*
 * This is a bad ripoff of the Little Man Computer, with extra steps
 * Instructions for use can be found in `README.md`
 * Rust port of the Python code
 */
use std::{
    collections::HashMap,
    io::{stdin, stdout, Stdin, Write},
    num::ParseIntError,
    str::SplitWhitespace,
    string::String,
    time::Instant,
};

const COMMENT_CHAR: char = ';';
const MAX_12B: u16 = 4095;
const MAX_13B: u16 = 8191;
const MAX_16B: u16 = 65535;

type LabelReturn = Result<(Vec<String>, Vec<u16>), (SyntaxError, Vec<String>)>;

enum SyntaxError {
    InvalidConfig,
    DuplicateLabel,
    UnknownLabel,
    InvalidLabel,
    UnknownOpcode,
    MissingOperand,
    InvalidOperand,
    BigOperand,
    TooManyTerms,
    LargeCode,
}

const KEYWORDS: [&str; 8] = ["HLT", "LDA", "STA", "ADD", "SUB", "JMP", "JMZ", "JMN"];
const EXTKEYWORDS: [&str; 6] = ["INP", "OUT", "OTA", "OTS", "OTB", "OTC"];
const BOOLS: [&str; 4] = ["TRUE", "1", "FALSE", "0"];

fn parse_line(raw_line: &str) -> Vec<&str> {
    /* Puts each term of line into a vector, filtering comments */
    let comment_filtered: &str = raw_line.split(COMMENT_CHAR).next().unwrap();
    let line_split: SplitWhitespace<'_> = comment_filtered.split_whitespace();
    let mut parsed_line: Vec<&str> = Vec::new();
    for term in line_split {
        if term.contains(';') {
            break;
        }
        parsed_line.push(term);
    }
    parsed_line
}

fn parse_code(raw_assembly: &str) -> Vec<Vec<&str>> {
    /* Parses text from lmcp file */
    let mut parsed_code: Vec<Vec<&str>> = Vec::new();

    for line in raw_assembly.split(['\n', '\t']) {
        let parsed_line: Vec<&str> = parse_line(line);
        if !parsed_line.is_empty() {
            parsed_code.push(parsed_line);
        }
    }
    parsed_code
}

fn check_configuration(parsed_code: &[Vec<&str>]) -> bool {
    /* Check if configuration lines are valid */
    if parsed_code[0].len() != 2
        || parsed_code[1].len() != 2
        || parsed_code[0][0] != "EXT"
        || parsed_code[1][0] != "RET"
        || !BOOLS.contains(&parsed_code[0][1].to_uppercase().as_str())
        || !BOOLS.contains(&parsed_code[1][1].to_uppercase().as_str())
    {
        return false;
    }
    true
}

fn get_configuration(code: &[Vec<&str>]) -> (bool, bool) {
    /* Get EXT and RET values */
    let trues: Vec<&str> = BOOLS[0..2].to_vec();
    let ext: bool = trues.contains(&code[0][1].to_uppercase().as_str());
    let ret: bool = trues.contains(&code[1][1].to_uppercase().as_str());
    (ext, ret)
}

fn get_labels(parsed_code: &[Vec<&str>], keywords: &[String]) -> LabelReturn {
    /*
     * Gets labels in code and their corresponding line number
     * If there is a duplicate label, returns (label, first occurence, offending occurrence)
     */
    let mut known_labels: Vec<String> = Vec::new();
    let mut known_labels_indexes: Vec<u16> = Vec::new();
    for (i, line) in parsed_code.iter().enumerate() {
        let first_term: String = line[0].to_ascii_uppercase();
        if known_labels.contains(&first_term) {
            let original: u16 = known_labels.iter().position(|r| r == &first_term).unwrap() as u16;
            return Err((
                SyntaxError::DuplicateLabel,
                vec![first_term, original.to_string(), (i as u16).to_string()],
            ));
        }
        if !keywords.contains(&first_term) {
            // Exclude DAT from labels
            if first_term == "DAT" {
                continue;
            }
            if first_term.chars().next().unwrap().is_numeric() {
                return Err((SyntaxError::InvalidLabel, vec![line[0].to_string()]));
            }
            known_labels.push(first_term);
            known_labels_indexes.push(i as u16);
        }
    }
    Ok((known_labels, known_labels_indexes))
}

fn check_syntax(
    parsed_code: &[Vec<&str>],
    keywords: &[String],
    op_size: u16,
    labels: &[String],
) -> Result<bool, (SyntaxError, Vec<String>)> {
    /* Check if rest of syntax is valid */
    let mut keywords: Vec<String> = keywords.to_vec();
    keywords.push(String::from("DAT")); // DAT is a keyword

    if parsed_code.len() > (op_size as usize) {
        return Err((SyntaxError::LargeCode, vec![]));
    }
    // Make vector of keywords that don't require operands
    let mut opless_keywords: Vec<String> = vec!["HLT".to_string(), "DAT".to_string()];
    for keyword in keywords.iter() {
        if EXTKEYWORDS.contains(&keyword.as_str()) {
            opless_keywords.push(keyword.to_string());
        }
    }

    for (i, line) in parsed_code.iter().enumerate() {
        match line.len() {
            1 => {
                // Expect operandless OPCODE
                let opcode: String = line[0].to_ascii_uppercase();
                if opless_keywords.contains(&opcode) {
                    continue;
                }
                if keywords.contains(&opcode) {
                    return Err((SyntaxError::MissingOperand, vec![i.to_string(), opcode]));
                }
                return Err((
                    SyntaxError::UnknownOpcode,
                    vec![i.to_string(), line[0].to_string()],
                ));
            }
            2 => {
                // Expect OPCODE OPERAND or LABEL OPCODE
                if keywords.contains(&line[0].to_ascii_uppercase()) {
                    // Must be OPCODE OPERAND
                    let operand: String = line[1].to_ascii_uppercase();
                    let operand_num: Result<usize, _> = operand.parse(); // Check if operand is a positive integer
                    if operand_num.is_err() {
                        // Operand is not a number
                        // Check if operand is a known label
                        if labels.contains(&operand) {
                            continue;
                        }
                        return Err((
                            SyntaxError::InvalidOperand,
                            vec![i.to_string(), line[1].to_string()],
                        ));
                    }
                    let opcode: String = line[0].to_ascii_uppercase();
                    let operand: usize = operand_num.unwrap();
                    if opcode == "DAT" {
                        // Operand max value is 2^16 - 1
                        if (MAX_16B as usize) < operand {
                            return Err((
                                SyntaxError::BigOperand,
                                vec![i.to_string(), operand.to_string(), MAX_16B.to_string()],
                            ));
                        }
                    } else if (op_size as usize) < operand {
                        // Check if operand is within bounds
                        return Err((
                            SyntaxError::BigOperand,
                            vec![i.to_string(), operand.to_string(), op_size.to_string()],
                        ));
                    }
                } else {
                    // Must be LABEL OPCODE
                    let opcode: String = line[1].to_ascii_uppercase();
                    if !keywords.contains(&opcode) {
                        return Err((SyntaxError::UnknownOpcode, vec![i.to_string()]));
                    }
                    let label: String = line[0].to_ascii_uppercase();
                    if !labels.contains(&label.clone()) {
                        return Err((SyntaxError::UnknownLabel, vec![i.to_string(), label]));
                    }
                    if opless_keywords.contains(&opcode) {
                        continue;
                    }
                    return Err((SyntaxError::MissingOperand, vec![i.to_string(), opcode]));
                }
            }
            3 => {
                // Expect LABEL OPCODE OPERAND
                let opcode: String = line[1].to_ascii_uppercase();
                if !keywords.contains(&opcode) {
                    return Err((
                        SyntaxError::UnknownOpcode,
                        vec![i.to_string(), line[1].to_string()],
                    ));
                }
                let operand: String = line[2].to_ascii_uppercase();
                if keywords.contains(&operand) {
                    return Err((
                        SyntaxError::InvalidOperand,
                        vec![i.to_string(), line[2].to_string()],
                    ));
                }
                let label: String = line[0].to_ascii_uppercase();
                if keywords.contains(&label) {
                    return Err((
                        SyntaxError::InvalidLabel,
                        vec![i.to_string(), line[0].to_string()],
                    ));
                }
                if !labels.contains(&label.clone()) {
                    return Err((SyntaxError::UnknownLabel, vec![i.to_string(), label]));
                }
            }
            _ => {
                return Err((
                    SyntaxError::TooManyTerms,
                    vec![i.to_string(), line.len().to_string()],
                ));
            }
        }
    }

    Ok(true)
}

fn spit_syntax_error(error: SyntaxError, args: Vec<String>) {
    use SyntaxError::*;
    match error {
        InvalidConfig => {
            eprintln!(
                "LMC Prime script does not start with:\n\
                1 | EXT (TRUE|FALSE)\n2 | RET (TRUE|FALSE)"
            );
        }
        DuplicateLabel => {
            eprintln!(
                "Duplicate label {} at lines {} and {}",
                args[0], args[1], args[2]
            );
        }
        UnknownLabel => {
            eprintln!("Unknown label {} at line {}", args[1], args[0]);
        }
        InvalidLabel => {
            eprintln!("Invalid label {} at line {}", args[1], args[0]);
        }
        UnknownOpcode => {
            eprintln!("Unknown opcode {} at line {}", args[1], args[0]);
        }
        MissingOperand => {
            eprintln!("Missing operand for opcode {} at line {}", args[1], args[0]);
        }
        InvalidOperand => {
            eprintln!("Invalid operand {} at line {}", args[1], args[0]);
        }
        BigOperand => {
            eprintln!(
                "Operand {} at line {} is too big (max {})",
                args[1], args[0], args[2]
            );
        }
        LargeCode => {
            eprintln!("Code is too large (max {})", args[0]);
        }
        TooManyTerms => {
            eprintln!(
                "Too many terms at line {} (expected 1-3, got {})",
                args[0], args[1]
            );
        }
    }
}

fn create_mailboxes(
    code: &[Vec<&str>],
    op_map: &HashMap<String, u8>,
    label_map: &HashMap<String, u16>,
    ext: bool,
    op_size: u16,
) -> Vec<u16> {
    let mut mailboxes: Vec<u16> = vec![0; op_size as usize];
    let shift: u16 = if ext { 12 } else { 13 };

    for (i, line) in code.iter().enumerate() {
        if line.contains(&"HLT") {
            continue;
        }

        let last_term_num: bool = line.last().unwrap().parse::<u16>().is_ok();

        let is_label_opcode: bool = !label_map
            .contains_key(&line.last().unwrap().to_ascii_uppercase())
            && line.len() == 2
            && !last_term_num;

        // Add operand
        if last_term_num {
            mailboxes[i] += line.last().unwrap().parse::<u16>().unwrap();
        } else if !is_label_opcode && !EXTKEYWORDS.contains(&line[0]) {
            mailboxes[i] += label_map
                .get(&line.last().unwrap().to_ascii_uppercase())
                .unwrap();
        }
        // Add opcode
        let opcode_index: usize = line.len()
            - (if 1 < line.len() && !is_label_opcode {
                2
            } else {
                1
            });
        mailboxes[i] += (*op_map
            .get(&line[opcode_index].to_ascii_uppercase())
            .unwrap() as u16)
            << shift;
    }
    mailboxes
}

fn execute(
    mailboxes: &mut [u16],
    ext: bool,
    ret: bool,
    op_map: &HashMap<String, u8>,
    printout: bool,
    code_length: u16,
) -> u16 {
    /* Runs the code from the first mailbox
    Returns accumulator's final value */
    let mut accumulator: u16 = 0;
    let mut program_counter: u16 = 0;
    let shift: u16 = if ext { 12 } else { 13 };

    loop {
        if printout && code_length > 0 {
            println!(
                "PC: {} | ACC: {} ({:.016b})",
                program_counter, accumulator, accumulator
            );
            print_mailbox_range(mailboxes, op_map, ext, 0, code_length, true)
        }
        let instruction_load: u16 = mailboxes[program_counter as usize];
        let opcode: u8 = (instruction_load >> shift) as u8;
        let operand: u16 = instruction_load & (if ext { MAX_12B } else { MAX_13B });

        match opcode {
            0 => {
                // HLT
                break;
            }
            1 => {
                // LDA
                accumulator = mailboxes[operand as usize];
            }
            2 => {
                // STA
                mailboxes[operand as usize] = accumulator;
            }
            3 => {
                // ADD
                accumulator += mailboxes[operand as usize];
            }
            4 => {
                // SUB
                accumulator -= mailboxes[operand as usize];
            }
            5 => {
                // JMP
                program_counter = operand;
                continue;
            }
            6 => {
                // JMZ
                if accumulator == 0 {
                    program_counter = operand;
                    continue;
                }
            }
            7 => {
                // JMN
                if accumulator >> 15 == 1 {
                    program_counter = operand;
                    continue;
                }
            }
            _ => {
                if ext {
                    match opcode {
                        8 => {
                            // INP
                            loop {
                                print!("Input: ");
                                stdout().flush().unwrap();
                                let mut input = String::new();
                                let stdin: Stdin = stdin();
                                match stdin.read_line(&mut input) {
                                    Ok(_) => {}
                                    Err(_) => {
                                        // User input empty
                                        println!(
                                            "Please enter a valid integer between 0 and {}",
                                            MAX_16B
                                        );
                                        continue;
                                    }
                                }
                                let input: Result<u16, ParseIntError> = input.trim().parse::<u16>();
                                if input.clone().is_ok() {
                                    accumulator = input.unwrap();
                                    break;
                                } else {
                                    println!(
                                        "Please enter a valid integer between 0 and {}",
                                        MAX_16B
                                    );
                                }
                            }
                        }
                        9 => {
                            // OUT
                            println!("{}", accumulator);
                        }
                        10 => {
                            // OTA
                            println!("{}", (accumulator % 256) as u8 as char);
                        }
                        11 => {
                            // OTS
                            let sign: i32 = (accumulator >> 15) as i32;
                            println!("{}", (accumulator as i32 & 32767) - (sign * 65536));
                        }
                        12 => {
                            // OTB
                            println!("{:016b}", accumulator);
                        }
                        13 => {
                            // OTC
                            let text: String = format!("{:016b}", accumulator);
                            println!("{}", text.replace('0', " "));
                        }
                        _ => {
                            // Unknown opcode
                            eprintln!("Invalid opcode: {}", opcode);
                            eprintln!("Make sure the program counter does not point to DAT");
                            break;
                        }
                    }
                } else {
                    // Unknown opcode
                    eprintln!("Invalid opcode: {}", opcode);
                    eprintln!("Make sure the program counter does not point to DAT");
                    break;
                }
            }
        }

        program_counter += 1;
    }

    if ret {
        println!("{}", accumulator);
    }
    accumulator
}

fn print_mailbox_range(
    mailboxes: &[u16],
    op_map: &HashMap<String, u8>,
    ext: bool,
    start: u16,
    end: u16,
    separate_ops: bool,
) {
    /* Prints contents of mailboxes in given range
    Content displayed in binary*/
    if !separate_ops {
        for address in start..end {
            println!("{:>4}: {:016b}", address, mailboxes[address as usize]);
        }
    } else {
        let mut reversed_op_map: HashMap<u8, String> = HashMap::new();
        for (key, value) in op_map {
            reversed_op_map.insert(*value, key.clone());
        }
        let spaces: (&str, &str) = if ext {
            ("            ", "   ")
        } else {
            ("             ", "  ")
        };
        println!("ADDRESS {} OPCODE {} OPERAND", spaces.0, spaces.1);

        for address in start..end {
            let opcode_len = if ext { 4 } else { 3 };
            let content = format!("{:016b}", mailboxes[address as usize]);

            let opcode: &str = &content[..opcode_len];
            let operand: &str = &content[opcode_len..];
            let int_opcode: u8 = u8::from_str_radix(opcode, 2).unwrap();
            let int_operand: u16 = u16::from_str_radix(operand, 2).unwrap();
            let read_opcode: Option<&String> = reversed_op_map.get(&int_opcode);
            let ops: String = match read_opcode {
                Some(real_opcode) => {
                    format!("{} ({}) {} ({})", real_opcode, opcode, operand, int_operand)
                }
                None => {
                    format!(
                        "DAT    {} ({})",
                        content,
                        u16::from_str_radix(content.as_str(), 2).unwrap()
                    )
                }
            };
            let bin_address: String = if ext {
                format!("{:012b}", address)
            } else {
                format!("{:013b}", address)
            };
            println!("{:>4} ({}): {}", address, bin_address, ops);
        }
    }
    println!()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        return eprintln!("Please provide a file path");
    }
    let file_path = &args[1];

    let start = Instant::now();
    let file_read: Result<String, std::io::Error> = std::fs::read_to_string(file_path);
    let file_contents: String = match file_read {
        Ok(contents) => {
            if contents.is_empty() {
                return eprintln!("File is empty");
            }
            contents
        }
        Err(_) => return eprintln!("File not found"),
    };

    // println!("{}", file_contents);

    let parsed_code = parse_code(&file_contents);
    // println!("{:?}", parsed_code);

    if !check_configuration(&parsed_code) {
        spit_syntax_error(SyntaxError::InvalidConfig, vec![])
    }
    let (ext, ret): (bool, bool) = get_configuration(&parsed_code);

    // Get keywords for instruction set
    let mut keywords: Vec<String> = Vec::new();
    let mut ext_keywords: Vec<String> = Vec::new();
    for keyword in KEYWORDS {
        keywords.push(String::from(keyword));
    }
    for keyword in EXTKEYWORDS {
        ext_keywords.push(String::from(keyword));
        if ext {
            keywords.push(String::from(keyword));
        }
    }

    let parsed_code: Vec<Vec<&str>> = parsed_code[2..].to_vec();

    let known_labels: Vec<String>;
    let known_labels_indexes: Vec<u16>;
    match get_labels(&parsed_code, &keywords) {
        Ok((labels, label_indexes)) => {
            (known_labels, known_labels_indexes) = (labels, label_indexes)
        }
        Err((error, args)) => return spit_syntax_error(error, args),
    }

    let op_size: u16 = if ext { MAX_12B } else { MAX_13B };
    match check_syntax(&parsed_code, &keywords, op_size, &known_labels) {
        Ok(_) => {}
        Err((error, args)) => return spit_syntax_error(error, args),
    }

    /* Code has been verified by this point */
    let mut op_map: HashMap<String, u8> = HashMap::with_capacity(keywords.len() + 1);
    op_map.insert("DAT".to_string(), 0);
    for (i, keyword) in keywords.iter().enumerate() {
        op_map.insert(keyword.to_string(), i as u8);
    }
    let mut label_map: HashMap<String, u16> = HashMap::with_capacity(op_size as usize);
    for (i, label) in known_labels.iter().enumerate() {
        label_map.insert(label.to_string(), known_labels_indexes[i]);
    }
    let mut mailboxes: Vec<u16> = create_mailboxes(&parsed_code, &op_map, &label_map, ext, op_size);

    let _: u16 = execute(
        &mut mailboxes,
        ext,
        ret,
        &op_map,
        false,
        parsed_code.len() as u16,
    );
    println!("{:?}", Instant::now().duration_since(start));
}
