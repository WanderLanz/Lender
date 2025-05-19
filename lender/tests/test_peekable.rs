use lender::prelude::{Lender, Lending};

struct Child<'lend> {
    stack: &'lend mut Vec<usize>,
    returned: bool,
}

impl<'lend> From<&'lend mut InnerParent> for Child<'lend> {
    fn from(parent: &'lend mut InnerParent) -> Self {
        let mut child = Self { stack: &mut parent.stack, returned: false };
        child.push_to_stack(0);
        child
    }
}

impl<'lend> Child<'lend> {
    fn push_to_stack(&mut self, row_id: usize) {
        println!("Pushing row {row_id} to stack at address {}", self.stack.as_ptr() as usize);
        self.stack.push(row_id);
        println!("Stack at address {} after push: {:?}", self.stack.as_ptr() as usize, self.stack);
    }

    fn pop_from_stack(&mut self) {
        println!("Popping row {} from stack at address {}", self.stack.last().unwrap(), self.stack.as_ptr() as usize);
        println!("Stack at address {} before pop: {:?}", self.stack.as_ptr() as usize, self.stack);
        self.stack.pop().unwrap();
        println!("Stack at address {} after pop: {:?}", self.stack.as_ptr() as usize, self.stack);
    }
}

impl<'lend2, 'lend> Lending<'lend2> for Child<'lend> {
    type Lend = &'lend2 [usize];
}

impl<'lend> Lender for Child<'lend> {
    fn next<'lend2>(&'lend2 mut self) -> Option<<Self as Lending<'lend2>>::Lend> {
        if self.returned {
            self.pop_from_stack();
            None
        } else {
            self.returned = true;
            Some(&self.stack)
        }
    }
}

struct InnerParent {
    /// The underlying data structure for the algorithm.
    stack: Vec<usize>,
    current_root_id: usize,
}

impl<'lend> Lending<'lend> for InnerParent {
    type Lend = Child<'lend>;
}

impl Lender for InnerParent {
    fn next<'lend>(&'lend mut self) -> Option<<Self as Lending<'lend>>::Lend> {
        while self.current_root_id < 5 {
            debug_assert!(
                self.stack.is_empty(),
                "Stack at address {} should be empty at the start of the inner child loop, but in parent is {:?}",
                self.stack.as_ptr() as usize,
                self.stack
            );

            self.current_root_id += 1;
            return Some(Child::from(self));
        }

        None
    }
}

pub struct Parent<'parent> {
    /// The underlying iterator.
    inner: lender::Peekable<'parent, InnerParent>,
}

impl<'parent> Iterator for Parent<'parent> {
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|vec| vec.stack.to_vec())
    }
}

#[test]
// This test at the the time of creation was causing an use-after-free error
// because of the wrong management of self-referencing data in Peekable.
fn test_peekable() {
    let inner_parent = InnerParent { stack: Vec::new(), current_root_id: 0 };
    let parent = Parent { inner: inner_parent.peekable() };
    let _results = parent.collect::<Vec<_>>();
}
