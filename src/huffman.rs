use crate::priority_queue::PriorityQueue;

struct HuffmanNode {
    weight: i32,
    children: Vec<HuffmanNode>,
}

/// Generates Huffman-encoded hints from an alphabet for N matches.
///
/// If n <= alphabet.len(), returns the first n characters.
/// Otherwise builds a Huffman tree to produce variable-length hints,
/// sorted by length (shortest first).
pub fn generate_hints(alphabet: &[String], n: usize) -> Vec<String> {
    if n == 0 {
        return Vec::new();
    }

    if n <= alphabet.len() {
        return alphabet[..n].to_vec();
    }

    let arity = alphabet.len();
    let mut queue = build_heap(n);

    let initial_branches = initial_number_of_branches(n, arity);
    let smallest = get_smallest(&mut queue, initial_branches);
    let new_node = new_node_from(smallest);
    queue.push(new_node.weight, new_node);

    while queue.size() > 1 {
        let smallest = get_smallest(&mut queue, arity);
        let new_node = new_node_from(smallest);
        queue.push(new_node.weight, new_node);
    }

    let root = queue.pop().unwrap();
    let mut result = Vec::new();
    traverse_tree(&root, &[], alphabet, &mut result);

    result.sort_by_key(|s| s.len());
    result
}

fn initial_number_of_branches(n: usize, arity: usize) -> usize {
    let mut result = arity;
    let n = n as i64;
    let arity = arity as i64;

    for _t in 1..=(n / arity + 1) {
        result = (n - _t * (arity - 1)) as usize;
        if result >= 2 && result <= arity as usize {
            break;
        }
        result = arity as usize;
    }

    result
}

fn build_heap(n: usize) -> PriorityQueue<HuffmanNode> {
    let mut queue = PriorityQueue::new();
    for i in 0..n {
        let weight = -(i as i32);
        queue.push(
            weight,
            HuffmanNode {
                weight,
                children: Vec::new(),
            },
        );
    }
    queue
}

fn get_smallest(queue: &mut PriorityQueue<HuffmanNode>, n: usize) -> Vec<HuffmanNode> {
    let count = n.min(queue.size());
    let mut result = Vec::with_capacity(count);
    for _ in 0..count {
        result.push(queue.pop().unwrap());
    }
    result
}

fn new_node_from(nodes: Vec<HuffmanNode>) -> HuffmanNode {
    let weight: i32 = nodes.iter().map(|n| n.weight).sum();
    HuffmanNode {
        weight,
        children: nodes,
    }
}

fn traverse_tree(
    node: &HuffmanNode,
    path: &[usize],
    alphabet: &[String],
    result: &mut Vec<String>,
) {
    if node.children.is_empty() {
        let hint: String = path.iter().map(|&i| alphabet[i].as_str()).collect();
        result.push(hint);
        return;
    }

    for (index, child) in node.children.iter().enumerate() {
        let mut new_path = path.to_vec();
        new_path.push(index);
        traverse_tree(child, &new_path, alphabet, result);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alphabet_a() -> Vec<String> {
        vec!["a", "s", "d", "f"]
            .into_iter()
            .map(String::from)
            .collect()
    }

    #[test]
    fn should_work_for_5() {
        let result = generate_hints(&alphabet_a(), 5);
        let expected: Vec<String> = vec!["s", "d", "f", "aa", "as"]
            .into_iter()
            .map(String::from)
            .collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn should_work_for_50() {
        let result = generate_hints(&alphabet_a(), 50);
        let expected: Vec<String> = vec![
            "aaa", "aas", "aad", "aaf", "asa", "ass", "asd", "asf", "ada", "ads", "add", "adf",
            "afa", "afd", "aff", "saa", "sas", "sad", "saf", "ssa", "sss", "ssd", "ssf", "sda",
            "sds", "sdd", "sdf", "sfa", "afsa", "afss", "afsd", "afsf", "sfsa", "sfss", "sfsd",
            "sfsf", "sfda", "sfds", "sfdd", "sfdf", "sffa", "sffs", "sffd", "sfffa", "sfffs",
            "sfffd", "sffffa", "sffffs", "sffffd", "sfffff",
        ]
        .into_iter()
        .map(String::from)
        .collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn returns_empty_for_zero() {
        let result = generate_hints(&alphabet_a(), 0);
        assert!(result.is_empty());
    }

    #[test]
    fn returns_first_n_when_n_lte_alphabet() {
        let result = generate_hints(&alphabet_a(), 3);
        let expected: Vec<String> = vec!["a", "s", "d"]
            .into_iter()
            .map(String::from)
            .collect();
        assert_eq!(result, expected);
    }
}
