//! In this example for the turingmachine-rs crate a turing machine is created which, when
//! supplied with two numbers `n` and `k` will determine whether `n` is divisable by `k`.
//!
//! Its usage is in the CLI by adding 2 args with numbers:
//! `cargo run --example divisibility -- 9 6`
//!
//! The turing machine starts with a tape in the shape:
//! ` _ 1^n _ 1^k _ ` and will output:
//!     - ` _ 1^n _ 1^k _ h _ 1 ` if k is a divisor of n
//!     - ` _ 1^n _ 1^k _ h _ 0 ` if k is __NOT__ a divisor of n
//!
//! here ` _ ` is a empty cell and ` 1^n ` is ` 1 ` repeated n times, and ` h ` is the halt symbol.

use std::fmt;
use turingmachine_rs::*;

#[derive(Clone, PartialEq)]
enum Alphabet {
    StartToken,
    Delta,
    Zero,
    One,
    MarkedOne,
    Halt,
}

impl fmt::Display for Alphabet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Alphabet::*;

        match self {
            StartToken => write!(f, "S"),
            Delta => write!(f, "_"),
            Zero => write!(f, "0"),
            One => write!(f, "1"),
            MarkedOne => write!(f, "!"),
            Halt => write!(f, "h"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
enum States {
    Start,
    MovingToCenter,
    CycleStart,
    CycleEnd,

    DivisibleReturn,
    NonDivisibleReturn,

    DivHalt,
    DivDelta,
    DivOutput,

    NonDivHalt,
    NonDivDelta,
    NonDivOutput,

    SearchingDen,
    SearchingNum,
    Found1InDenominator,
    Found1InNuminator,

    CheckingDivByNull,

    CheckingLeftovers,
    FoundLeftovers,

    NoLeftoversP1,
    NoLeftoversP2,

    LeftOverP1,
    LeftOverP2,
    
    DivByNull,
    InvalidSyntax,

    Done
}

impl TuringStates<Alphabet> for States {
    fn step(&self, t: Alphabet) -> (Self, Alphabet, Move) {
        use Alphabet::*;
        use States::*;

        match self {
            Start => match t {
                Delta => (MovingToCenter, t, Move::Right),
                _ => (Start, t, Move::Right),
            },

            MovingToCenter => match t {
                One => (MovingToCenter, t, Move::Right),
                Delta => (CheckingDivByNull, t, Move::Right),
                _ => (InvalidSyntax, t, Move::Stay),
            },

            CheckingDivByNull => match t {
                Delta => (DivByNull, t, Move::Stay),
                _ => (CycleStart, t, Move::Left),
            },

            CycleStart => (CheckingLeftovers, t, Move::Left),

            CheckingLeftovers => match t {
                Delta => (NoLeftoversP1, t, Move::Right),
                One => (FoundLeftovers, t, Move::Right),
                _ => (CheckingLeftovers, t, Move::Left),
            },

            FoundLeftovers => match t {
                Delta => (SearchingDen, t, Move::Right),
                _ => (FoundLeftovers, t, Move::Right),
            },

            SearchingDen => match t {
                Delta => (CycleEnd, t, Move::Left),
                One => (Found1InDenominator, MarkedOne, Move::Left),
                _ => (SearchingDen, t, Move::Right),
            },

            Found1InDenominator => match t {
                Delta => (SearchingNum, t, Move::Left),
                _ => (Found1InDenominator, t, Move::Left),
            },

            SearchingNum => match t {
                Delta => (LeftOverP1, t, Move::Right),
                One => (Found1InNuminator, MarkedOne, Move::Right),
                _ => (SearchingNum, t, Move::Left),
            },

            Found1InNuminator => match t {
                Delta => (SearchingDen, t, Move::Right),
                _ => (Found1InNuminator, t, Move::Right),
            },

            CycleEnd => match t {
                Delta => (CycleStart, t, Move::Stay),
                MarkedOne => (CycleEnd, One, Move::Left),
                _ => panic!("Something went wrong"),
            },

            LeftOverP1 => match t {
                Delta => (LeftOverP2, t, Move::Right),
                MarkedOne => (LeftOverP1, One, Move::Right),
                _ => (LeftOverP1, t, Move::Right),
            }

            LeftOverP2 => match t {
                Delta => (NonDivisibleReturn, t, Move::Right),
                MarkedOne => (LeftOverP2, One, Move::Right),
                _ => (LeftOverP2, t, Move::Right),
            }

            NoLeftoversP1 => match t {
                Delta => (NoLeftoversP2, t, Move::Right),
                MarkedOne => (NoLeftoversP1, One, Move::Right),
                _ => (NoLeftoversP1, t, Move::Right),
            }

            NoLeftoversP2 => match t {
                Delta => (DivisibleReturn, t, Move::Right),
                MarkedOne => (NoLeftoversP2, One, Move::Right),
                _ => (NoLeftoversP2, t, Move::Right),
            }
            
            DivisibleReturn => (DivHalt, t, Move::Stay),            
            DivHalt => (DivDelta, Halt, Move::Right),            
            DivDelta => (DivOutput, Delta, Move::Right),            
            DivOutput => (Done, One, Move::Right),            

            NonDivisibleReturn => (NonDivHalt, t, Move::Stay),            
            NonDivHalt => (NonDivDelta, Halt, Move::Right),            
            NonDivDelta => (NonDivOutput, Delta, Move::Right),            
            NonDivOutput => (Done, Zero, Move::Right),            

            _ => (InvalidSyntax, t, Move::Stay),
        }
    }
}

fn main() {
    let n = std::env::args()
        .nth(1)
        .expect("No first argument given. Usage: cargo run --example divisibility -- <int> <int>");
    let k = std::env::args()
        .nth(2)
        .expect("No second argument given. Usage: cargo run --example divisibility -- <int> <int>");

    let n = n
        .parse::<usize>()
        .expect("First argument is not a positive integer");
    let k = k
        .parse::<usize>()
        .expect("Second argument is not a positive integer");

    println!("n: {}, k: {}", n, k);

    let mut ns = vec![Alphabet::One; n];
    let mut ks = vec![Alphabet::One; k];

    let mut delta = vec![Alphabet::Delta];

    let mut initial = Vec::new();

    initial.append(&mut delta.clone());
    initial.append(&mut ns);
    initial.append(&mut delta.clone());
    initial.append(&mut ks);
    initial.append(&mut delta);

    let tape = TuringTape::new(Alphabet::Delta, Alphabet::StartToken, initial);
    println!("Tape: {}", tape);

    println!("\n\nRunning...\n\n");

    use States::*;
    let end_state = tape.debug_run_states(
        Start,
        vec![InvalidSyntax, DivByNull, Done],
    );

    println!("\n\nFinished Running...\n\n");

    println!("Tape: {}", tape);
    println!("Endstate: {:?}", end_state);
}