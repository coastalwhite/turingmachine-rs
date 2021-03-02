//! In this integration test a turing machine is created which replaces all the Ones in the input
//! with zeros.

use turingmachine_rs::*;

/// The Alphabet Used
#[derive(PartialEq, Clone, Debug)]
enum Alphabet {
    Delta,
    Zero,
    One,
}

/// All the different states
#[derive(PartialEq, Debug)]
enum States {
    Start,
    Started,
    ValidEnd,
}

/// The implementation for the states
impl TuringStates<Alphabet> for States {
    fn int_step(&mut self, cursor_token: Alphabet) -> (Option<Alphabet>, Move) {
        use Alphabet::*;
        use States::*;

        match self {
            Start => {
                *self = Started;
                (None, Move::Right)
            },

            ValidEnd => panic!("ValidEnd should be including in the end states and shouldn't be the initial state."),

            Started => {
                match cursor_token {
                    Zero => (None, Move::Right),
                    One => (Some(Zero), Move::Right),
                    Delta => {
                        *self = ValidEnd;
                        (None, Move::Stay)
                    }
                }
            },
        }
    }
}

#[test]
fn replace_ones_proper_output() {
    use turingmachine_rs::TuringStates;
    use Alphabet::*;
    use States::*;
    assert_eq!(
        States::run_until_end(
            Start,
            vec![ValidEnd],
            Delta,
            Delta,
            vec![Zero, Zero, Zero, One, One]
        ),
        (ValidEnd, vec![Delta, Zero, Zero, Zero, Zero, Zero, Delta])
    );
}
