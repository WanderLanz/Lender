use lender::prelude::{Lender, Lending};

struct Child<'lend> {
    data: &'lend mut Data,
    found_circuit: bool,
}

impl<'lend> From<&'lend mut InnerParent> for Child<'lend> {
    fn from(parent: &'lend mut InnerParent) -> Self {
        let mut circuit_search = Self {
            data: &mut parent.data,
            found_circuit: false,
        };
        circuit_search.push_to_stack(0);
        circuit_search
    }
}

impl<'lend> Child<'lend> {
    fn push_to_stack(&mut self, row_id: usize) {
        println!(
            "Pushing row {row_id} to stack at address {}",
            self.data.stack.as_ptr() as usize
        );
        self.data.stack.push(row_id);
        println!(
            "Stack at address {} after push: {:?}",
            self.data.stack.as_ptr() as usize,
            self.data.stack
        );
    }

    fn pop_from_stack(&mut self) {
        println!(
            "Popping row {} from stack at address {}",
            self.data.stack.last().unwrap(),
            self.data.stack.as_ptr() as usize
        );
        println!(
            "Stack at address {} before pop: {:?}",
            self.data.stack.as_ptr() as usize,
            self.data.stack
        );
        self.data.stack.pop().unwrap();
        println!(
            "Stack at address {} after pop: {:?}",
            self.data.stack.as_ptr() as usize,
            self.data.stack
        );
    }
}

impl<'lend2, 'lend> Lending<'lend2> for Child<'lend> {
    type Lend = &'lend2 [usize];
}

impl<'lend> Lender for Child<'lend> {
    fn next<'lend2>(&'lend2 mut self) -> Option<<Self as Lending<'lend2>>::Lend> {
        if self.found_circuit {
            self.pop_from_stack();
            None
        } else {
            self.found_circuit = true;
            Some(&self.data.stack)
        }
    }
}

struct Data {
    /// The stack of nodes in the current circuit.
    stack: Vec<usize>,
}

/// Iterator for Johnson's algorithm.
struct InnerParent {
    /// The underlying data structure for the algorithm.
    data: Data,
    current_root_id: usize,
}

impl<'lend> Lending<'lend> for InnerParent {
    type Lend = Child<'lend>;
}

impl Lender for InnerParent {
    fn next<'lend>(&'lend mut self) -> Option<<Self as Lending<'lend>>::Lend> {
        while self.current_root_id < 5 {
            assert!(
                self.data.stack.is_empty(),
                "Stack at address {} should be empty at the start of the circuit search, but in parent is {:?}",
                self.data.stack.as_ptr() as usize,
                self.data.stack
            );

            self.current_root_id += 1;
            return Some(Child::from(self));
        }

        None
    }
}

/// Johnson's algorithm for finding all cycles in a sparse matrix.
pub struct Parent<'parent> {
    /// The underlying iterator.
    inner: lender::Flatten<'parent, InnerParent>,
}

impl<'matrix> Iterator for Parent<'matrix> {
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|circuit| circuit.to_vec())
    }
}


#[test]
fn test_flatten() {
	let inner_parent = InnerParent {
        data: Data { stack: Vec::new() },
        current_root_id: 0,
    };
    let parent = Parent {
        inner: inner_parent.flatten(),
    };
    let results = parent.collect::<Vec<_>>();
}