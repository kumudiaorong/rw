use std::io::Read;

use crate::State;
fn perm<F>(collection: &mut Vec<char>, start: usize, v: &mut F)
where
    F: FnMut(&mut Vec<char>, usize, usize, State),
{
    if start == collection.len() - 1 {
        v(collection, start, start, State::Done);
    } else {
        v(collection, start, start, State::NotRepeated);
        perm(collection, start + 1, v);
        v(collection, start, start, State::Back);
        for i in start + 1..collection.len() {
            if collection[i] != collection[i - 1] {
                v(collection, i, start, State::Selected);
                collection.swap(start, i);
                v(collection, i, start, State::Swapped);
                perm(collection, start + 1, v);
                v(collection, i, start, State::Back);
                collection.swap(start, i);
            }
        }
    }
}

pub fn run<F>(v: &mut F)
where
    F: FnMut(&mut Vec<char>, usize, usize, State),
{
    let mut collection = Vec::new();
    let mut file = std::fs::File::open("input.txt").unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    let mut lines = content.lines();
    let n = lines.next().unwrap().parse::<usize>().unwrap();
    for line in lines {
        for c in line.chars() {
            collection.push(c);
        }
    }
    collection.resize(n, '-');
    v(&mut collection, 0, 0, State::Start);
    collection.sort();
    v(&mut collection, 0, 0, State::Sorted);
    perm(&mut collection, 0, v);
}
