use std::collections::BTreeMap;

/// A priority queue implemented as a BTreeMap of priority -> items.
/// Higher numeric priority values are popped first.
pub struct PriorityQueue<T> {
    q: BTreeMap<i32, Vec<T>>,
}

impl<T> PriorityQueue<T> {
    pub fn new() -> Self {
        Self {
            q: BTreeMap::new(),
        }
    }

    pub fn push(&mut self, priority: i32, item: T) {
        self.q.entry(priority).or_default().push(item);
    }

    pub fn pop(&mut self) -> Option<T> {
        let &p = self.q.keys().last()?;
        let items = self.q.get_mut(&p)?;
        let item = items.remove(0);
        if items.is_empty() {
            self.q.remove(&p);
        }
        Some(item)
    }

    pub fn size(&self) -> usize {
        self.q.values().map(|v| v.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pops_in_priority_order_highest_first() {
        let test_data = vec![
            (3, "Clear drains"),
            (6, "drink tea"),
            (5, "Make tea"),
            (4, "Feed cat"),
            (7, "eat biscuit"),
            (2, "Tax return"),
            (1, "Solve RC tasks"),
        ];

        let mut pq = PriorityQueue::new();
        for (priority, item) in test_data {
            pq.push(priority, item);
        }

        let mut results = Vec::new();
        while let Some(item) = pq.pop() {
            results.push(item);
        }

        let expected = vec![
            "eat biscuit",
            "drink tea",
            "Make tea",
            "Feed cat",
            "Clear drains",
            "Tax return",
            "Solve RC tasks",
        ];

        assert_eq!(results, expected);
    }
}
