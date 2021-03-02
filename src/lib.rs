//! A simulation crate for Turing Machines
#![warn(missing_docs)]

use std::cell::RefCell;
use std::fmt;
use std::rc::{Rc, Weak};

/// A struct representing a node in a linked list
#[derive(Clone)]
struct Node<Alphabet> {
    /// The previous node
    prev: Option<Rc<Node<Alphabet>>>,
    /// The next node
    next: RefCell<Option<Weak<Node<Alphabet>>>>,
    /// The data contained in the node
    data: RefCell<Alphabet>,
}

impl<Alphabet: Clone> Node<Alphabet> {
    /// Create a new node with a data and possibly a previous node
    fn new(data: Alphabet, prev: Option<Rc<Node<Alphabet>>>) -> Node<Alphabet> {
        Node {
            prev,
            next: RefCell::new(None),
            data: RefCell::new(data),
        }
    }

    /// Fetch previous node
    fn prev(&self) -> Option<Rc<Node<Alphabet>>> {
        self.prev.clone()
    }

    /// Fetch next node
    fn next(&self) -> Option<Weak<Node<Alphabet>>> {
        self.next.borrow().clone()
    }

    /// Replace next with possible new next node
    fn replace_next(
        &self,
        new_value: Option<Weak<Node<Alphabet>>>,
    ) -> Option<Weak<Node<Alphabet>>> {
        self.next.replace(new_value)
    }

    /// Fetch the data contained in node
    fn get(&self) -> Alphabet {
        self.data.borrow().clone()
    }

    /// Replace the data contained in the node
    fn replace(&self, new_value: Alphabet) -> Alphabet {
        self.data.replace(new_value)
    }
}

/// A possibly theorically infinite TuringTape
pub struct TuringTape<Alphabet> {
    /// The alphabet token put at empty spaces
    empty: Alphabet,
    /// The last node currently saved
    last: RefCell<Rc<Node<Alphabet>>>,
    /// The cursor point
    cursor: RefCell<Rc<Node<Alphabet>>>,
}

impl<Alphabet: fmt::Display + Clone> fmt::Display for TuringTape<Alphabet> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "|")?;
        let mut s = String::new();

        let mut head: Rc<Node<Alphabet>> = self.last.borrow().clone();

        loop {
            let backup = s.clone();
            if Rc::ptr_eq(&self.cursor.borrow(), &head) {
                s = format!("> {} <|", head.get());
            } else {
                s = format!("  {}  |", head.get());
            }
            s.push_str(&backup);

            if let Some(new_head) = head.prev.clone() {
                head = new_head;
            } else {
                break;
            }
        }

        write!(f, "{}", s)
    }
}

impl<Alphabet: Clone> TuringTape<Alphabet> {
    /// Initialize a new TuringTape with:
    ///
    /// - __empty:__ The token put at empty tape cells
    /// - __start:__ The token put in the first cell
    /// - __initial:__ An vector of tokens to be put after the start token
    pub fn new(empty: Alphabet, start: Alphabet, initial: Vec<Alphabet>) -> TuringTape<Alphabet> {
        let fst_node = Rc::new(Node::new(start, None));
        let tape = TuringTape {
            empty,
            last: RefCell::new(fst_node.clone()),
            cursor: RefCell::new(fst_node),
        };

        initial.into_iter().for_each(|token| {
            tape.append(token);
        });

        tape
    }

    /// Append a new token to the turing tape
    fn append(&self, token: Alphabet) -> Rc<Node<Alphabet>> {
        let new_node = Rc::new(Node::new(token, Some(self.last.borrow().clone())));
        self.last
            .borrow()
            .replace_next(Some(Rc::downgrade(&new_node)));
        self.last.replace(new_node.clone());
        new_node
    }

    /// Fetch the token at the cursor
    pub fn get_cursor(&self) -> Alphabet {
        self.cursor.borrow().clone().get()
    }

    /// Set the token at the cursor and return the old token
    pub fn set_cursor(&self, value: Alphabet) -> Alphabet {
        self.cursor.borrow().clone().replace(value)
    }

    /// Make the cursor go one cell to the right
    pub fn step_right(&self) -> Alphabet {
        let new_cursor = match self.cursor.borrow().clone().next() {
            Some(next) => next.clone().upgrade().expect("Unable to upgrade"),
            None => self.append(self.empty.clone()),
        };

        self.cursor.replace(new_cursor);
        self.get_cursor()
    }

    /// Make the cursor go one cell to the left
    ///
    /// Will panic if one goes off the tape.
    pub fn step_left(&self) -> Alphabet {
        let new_cursor = match self.cursor.borrow().clone().prev() {
            Some(prev) => prev.clone(),
            None => panic!("Went left side of the tape!"),
        };

        self.cursor.replace(new_cursor);
        self.get_cursor()
    }

    /// Runs from start state until one of the end states has been reached.
    /// Will return the end state.
    pub fn run_states<S: TuringStates<Alphabet> + PartialEq>(
        &self,
        mut start_state: S,
        end_states: Vec<S>,
    ) -> S {
        while !end_states.contains(&start_state) {
            start_state.step(self);
        }

        start_state
    }
}

impl<Alphabet: Clone> From<TuringTape<Alphabet>> for Vec<Alphabet> {
    fn from(tape: TuringTape<Alphabet>) -> Vec<Alphabet> {
        let cursor_initial = tape.cursor.borrow().clone();
        let mut v = Vec::new();

        // Move all the way to the left
        while tape.cursor.borrow().clone().prev().is_some() {
            tape.step_left();
        }

        // Move all the way back to the right adding the cursor
        // to the vector along the way
        while tape.cursor.borrow().clone().next().is_some() {
            v.push(tape.get_cursor());
            tape.step_right();
        }

        // Add the last element
        v.push(tape.get_cursor());

        // Set the cursor back to it's previous position
        let mut cursor = tape.cursor.borrow_mut();
        *cursor = cursor_initial;

        v
    }
}

/// Define the movement direction
pub enum Move {
    /// Move left one cell
    Left,
    /// Move right one cell
    Right,
}

/// A trait that implements the behaviour for turing states
pub trait TuringStates<Alphabet: Clone>: Sized + PartialEq {
    /// The internal step function
    /// Output the new state, token at current cursor position, and move of the cursor position
    fn int_step(&self, current_token: Alphabet) -> (Option<Self>, Option<Alphabet>, Option<Move>);

    /// Execute one step of the turing machine
    fn step(&mut self, tape: &TuringTape<Alphabet>) {
        let (opt_state, opt_replace, opt_move) = self.int_step(tape.get_cursor());

        // Update the current state
        if let Some(state) = opt_state {
            *self = state;
        }

        // Update cursor token
        if let Some(replace) = opt_replace {
            tape.set_cursor(replace);
        }

        // Update cursor position
        if let Some(mv) = opt_move {
            match mv {
                Move::Left => {
                    tape.step_left();
                }
                Move::Right => {
                    tape.step_right();
                }
            };
        }
    }

    /// Run this turing machine from a start state, until it eaches a final state.
    /// Will return a tuple containing the end_state and a vector of the memory state.
    fn run_until_end(
        start_state: Self,
        end_states: Vec<Self>,
        empty_token: Alphabet,
        start_token: Alphabet,
        initial_state: Vec<Alphabet>,
    ) -> (Self, Vec<Alphabet>) {
        let tape = TuringTape::new(empty_token, start_token, initial_state);
        let end_state = tape.run_states(start_state, end_states);
        (end_state, tape.into())
    }
}

#[cfg(test)]
mod tests {
    #[derive(PartialEq, Clone, Debug)]
    pub enum Bit {
        Delta,
        Zero,
        One,
    }

    impl fmt::Display for Bit {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            use Bit::*;

            match self {
                Delta => write!(f, "_"),
                Zero => write!(f, "0"),
                One => write!(f, "1"),
            }
        }
    }

    use super::*;

    #[test]
    fn fetch_cursor() {
        use Bit::*;
        assert_eq!(TuringTape::new(Delta, Zero, vec![]).get_cursor(), Zero);
        assert_eq!(TuringTape::new(Delta, One, vec![]).get_cursor(), One);
        assert_eq!(TuringTape::new(Delta, Delta, vec![]).get_cursor(), Delta);
        assert_eq!(TuringTape::new(Delta, Zero, vec![Delta]).get_cursor(), Zero);
        assert_eq!(TuringTape::new(Delta, One, vec![Delta]).get_cursor(), One);
        assert_eq!(
            TuringTape::new(Delta, Delta, vec![Delta]).get_cursor(),
            Delta
        );
    }

    #[test]
    fn set_cursor() {
        use Bit::*;
        let tape = TuringTape::new(Delta, Delta, vec![Zero, One, Zero]);
        assert_eq!(tape.get_cursor(), Delta);
        tape.set_cursor(One);
        assert_eq!(tape.get_cursor(), One);
        tape.set_cursor(Zero);
        assert_eq!(tape.get_cursor(), Zero);
    }

    #[test]
    fn turing_tape_new() {
        use Bit::*;
        TuringTape::new(
            Delta,
            Delta,
            vec![Zero, One, One, One, Zero, One, One, One, Zero],
        );
    }

    #[test]
    fn turing_into_vec() {
        use Bit::*;
        let tape = TuringTape::new(
            Delta,
            Delta,
            vec![Zero, One, One, One, Zero, One, One, One, Zero],
        );
        assert_eq!(
            <Vec<Bit>>::from(tape),
            vec![Delta, Zero, One, One, One, Zero, One, One, One, Zero]
        );
    }

    #[test]
    fn turing_stepping() {
        use Bit::*;
        let tape = TuringTape::new(
            Delta,
            Delta,
            vec![Zero, One, One, One, Zero, One, One, One, Zero],
        );

        assert_eq!(tape.get_cursor(), Delta);
        tape.step_right();
        assert_eq!(tape.get_cursor(), Zero);
        tape.step_left();
        assert_eq!(tape.get_cursor(), Delta);

        tape.step_right();
        tape.step_right();
        assert_eq!(tape.get_cursor(), One);
        tape.step_right();
        assert_eq!(tape.get_cursor(), One);
        tape.step_right();
        assert_eq!(tape.get_cursor(), One);
        tape.step_right();
        assert_eq!(tape.get_cursor(), Zero);

        assert_eq!(tape.step_right(), tape.get_cursor());
        assert_eq!(tape.step_right(), tape.get_cursor());
        assert_eq!(tape.step_right(), tape.get_cursor());
        assert_eq!(tape.step_right(), tape.get_cursor());
    }
}
