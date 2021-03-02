//! In this integration test a turing machine is created which determines whether the input
//! contains exactly two `One`s.

use turingmachine_rs::*;

/// The Alphabet Used
#[derive(PartialEq, Clone, Debug)]
enum Alphabet {
    Delta,
    Zero,
    One,
}

/// All the different states
#[derive(PartialEq, Debug, Clone, Copy)]
enum States {
    Start,
    FoundNone,
    FoundFirst,
    FoundSecond,
    FoundMore,
    InvalidEnd,
    ValidEnd,
}

/// The implementation for the states
impl TuringStates<Alphabet> for States {
    fn step(&self, t: Alphabet) -> (Self, Alphabet, Move) {
        use Alphabet::*;
        use States::*;

        match self {
            Start => (FoundNone, t, Move::Right),

            InvalidEnd => panic!("ValidEnd should be including in the end states and shouldn't be the initial state."),

            _ => {
                match t {
                    Zero => (*self, t, Move::Right),

                    One => (match self {
                        FoundNone => FoundFirst,
                        FoundFirst => FoundSecond,
                        FoundSecond => FoundMore,
                        FoundMore => FoundMore,
                        _ => panic!("Unreachable"),
                    }, t, Move::Right),

                    Delta => (match self {
                        FoundSecond => ValidEnd,
                        _ => InvalidEnd,
                    }, t, Move::Stay)
                }
            },
        }
    }
}

#[test]
fn exactly_two_proper_output() {
    use turingmachine_rs::TuringStates;
    use Alphabet::*;
    use States::*;
    assert_eq!(
        States::run_until_end(
            Start,
            vec![FoundMore, InvalidEnd, ValidEnd],
            Delta,
            Delta,
            vec![Zero, Zero, Zero, One, One]
        ),
        (ValidEnd, vec![Delta, Zero, Zero, Zero, One, One, Delta])
    );
}
