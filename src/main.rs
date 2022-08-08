use std::fs;
use std::io::Read;

use clap::Parser;
use itertools::Itertools;

use anyhow::Result;

/// reads a file into a u8 vector
/// - `prefix_length` : the prefix in bytes to read from `filename`. 0 means to read the entire file
pub fn file2byte_vector(filename: &str, prefix_length: Option<u64>) -> Result<Vec<u8>> {
    let path = std::path::Path::new(filename);
    let mut f = fs::File::open(&path).expect("no file found");
    let metadata = fs::metadata(&path).expect("unable to read metadata");
    let num_file_bytes = metadata.len();
    let buffer_length = prefix_length
        .map(|v| v.min(num_file_bytes))
        .unwrap_or(num_file_bytes);
    let buffer_length = usize::try_from(buffer_length)?;
    let mut buffer = Vec::with_capacity(buffer_length);
    buffer.resize(buffer_length as usize, 0u8);
    f.read_exact(&mut buffer)?;
    Ok(buffer)
}

#[derive(Debug, Copy, Clone)]
struct StackElement {
    text_pos: usize,
    period: usize,
}

fn successor_element(text: &[u8], start: usize, value: u8) -> Option<usize> {
    if start >= text.len() {
        return None;
    }
    let mut list = leftmost_distinct_characters(&text[start..]);
    list.find_map(|v| {
        if text[start + v] >= value {
            Some(start + v)
        } else {
            None
        }
    })
}

fn leftmost_distinct_characters(text: &[u8]) -> impl Iterator<Item = usize> {
    let mut charmap = [usize::MAX; u8::MAX as usize];
    for (i, c) in text.iter().enumerate().rev() {
        charmap[*c as usize] = i;
    }
    charmap
        .into_iter()
        .filter(|&x| x != usize::MAX)
        // unsorted stable valid here? or does it have to be stable?
        .sorted_unstable_by(|a, b| text[*a].partial_cmp(&text[*b]).unwrap())
}

fn subsequence(text: &[u8], stack: &[StackElement]) -> Vec<u8> {
    stack.iter().map(|el| text[el.text_pos]).collect()
}

fn longest_lyndon_subsequence(text: &[u8]) -> Vec<StackElement> {
    let mut larray = vec![usize::MAX; text.len() + 1];

    let mut longest_lyndon_subsequence = Vec::new();

    let mut stack = Vec::new();
    let mut lastchildedgelabel = 0u8;
    let mut upwardmove = false;
    for starting_position in leftmost_distinct_characters(text) {
        stack.push(StackElement {
            text_pos: starting_position,
            period: 1,
        });
        if longest_lyndon_subsequence.is_empty() {
            longest_lyndon_subsequence = stack.clone();
        }
        while !stack.is_empty() {
            let top = stack.last().unwrap();
            let immature_character = text[stack[stack.len() - top.period as usize].text_pos];
            let compare_char = if upwardmove {
                lastchildedgelabel
            } else {
                immature_character
            };
            match successor_element(text, top.text_pos + 1, compare_char) {
                None => {
                    upwardmove = true;
                    lastchildedgelabel = text[top.text_pos] + 1;
                    stack.pop();
                }
                Some(i) => {
                    assert!(top.text_pos < i);
                    assert!(compare_char <= text[i]);
                    let subsequence_length = stack.len() + 1;
                    if larray[subsequence_length] < i {
                        upwardmove = true;
                        lastchildedgelabel = text[i] + 1;
                    } else {
                        let new_period = if immature_character == text[i] {
                            top.period
                        } else {
                            subsequence_length
                        };

                        stack.push(StackElement {
                            text_pos: i,
                            period: new_period,
                        });
                        if new_period == subsequence_length {
                            //@ only update larray if we have a Lyndon subsequence
                            larray[subsequence_length] = i;
                            if longest_lyndon_subsequence.len() < subsequence_length {
                                longest_lyndon_subsequence = stack.clone();
                            }
                        }
                        upwardmove = false;
                    }
                }
            }
        }
    }
    longest_lyndon_subsequence
}

#[test]
fn test_lyndon_subsequence() {
    fn check_subsequence(text: &[u8], result: &[u8]) {
        assert_eq!(subsequence(text, &longest_lyndon_subsequence(text)), result);
    }
    check_subsequence(b"bccadbaccbcd", b"bccbccbcd");
    check_subsequence(b"bccadbaccbc", b"abaccbc");
    check_subsequence(b"bccadbaccb", b"abaccb");
    check_subsequence(b"bccadbacc", b"bccdcc");
    check_subsequence(b"a", b"a");
    check_subsequence(b"aa", b"a");
    check_subsequence(b"aaa", b"a");
    check_subsequence(b"aaab", b"aaab");
    check_subsequence(b"aaaba", b"aaab");
}

/// Computes the longest Lyndon subsequence
#[derive(Parser, Debug)]
struct Args {
    /// input filename
    #[clap(short, long)]
    filename: String,

    /// the number of characters to read from the input file
    #[clap(short, long)]
    prefix: Option<u64>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let text = file2byte_vector(&args.filename, args.prefix)?;

    let stack_subsequence = longest_lyndon_subsequence(&text);
    println!(
        "{}",
        std::str::from_utf8(&subsequence(&text, &stack_subsequence)).unwrap()
    );
    Ok(())
}
