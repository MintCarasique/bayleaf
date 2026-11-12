//! Which of the right rail's collapsible controls is open. At most one at a time.

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Accordion {
    open: Option<usize>,
}

impl Accordion {
    /// Tap on control `i`: opens it (closing any other) or closes it if it was open.
    /// Returns the control that is open afterwards.
    pub fn toggle(&mut self, i: usize) -> Option<usize> {
        self.open = if self.open == Some(i) { None } else { Some(i) };
        self.open
    }

    pub fn close(&mut self) {
        self.open = None;
    }

    pub fn open(&self) -> Option<usize> {
        self.open
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opens_one_at_a_time() {
        let mut a = Accordion::default();
        assert_eq!(a.toggle(0), Some(0));
        assert_eq!(a.toggle(1), Some(1));
        assert_eq!(a.open(), Some(1));
    }

    #[test]
    fn second_tap_closes() {
        let mut a = Accordion::default();
        a.toggle(2);
        assert_eq!(a.toggle(2), None);
    }

    #[test]
    fn close_resets() {
        let mut a = Accordion::default();
        a.toggle(0);
        a.close();
        assert_eq!(a.open(), None);
    }
}
