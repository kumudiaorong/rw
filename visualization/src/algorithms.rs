// fn perm(
//     collection: &mut Vec<i32>,
//     start: usize,
//     end: usize,
//     v: fn(all: &mut Vec<i32>, sel: &mut [bool; 26], sel_idx: usize, all_idx: usize, repeat: bool),
// ) {
//     if start == end {
//         println!("{:?}", collection);
//     } else {
//         let mut flag = [false; 26];
//         for i in start..end {
//             let idx = collection[i] as usize - 'a' as usize;
//             if flag[idx] {
//                 v(collection, &mut flag, i, start, true);
//                 continue;
//             }
//             v(collection, &mut flag, i, start, false);
//             flag[idx] = true;
//             collection.swap(start, i);
//             v(collection, &mut flag, i, start, false);
//             perm(collection, start + 1, end, v);
//             collection.swap(start, i);
//         }
//     }
// }

use std::io::Read;

// pub fn run(collection: &mut Vec<i32>) {
//     perm(
//         collection,
//         0,
//         collection.len(),
//         |all, sel, sel_idx, all_idx, repeat| {
//             println!(
//                 "all: {:?}, sel: {:?}, sel_idx: {}, all_idx: {}, repeat: {}",
//                 all, sel, sel_idx, all_idx, repeat
//             );
//         },
//     );
// }
use crate::State;
fn perm<F>(collection: &mut Vec<char>, start: usize, v: &mut F)
where
    F: FnMut(&mut Vec<char>, usize, usize, State),
{
    if start == collection.len() - 1 {
        v(collection, start, start, State::Done);
    } else {
        for i in start..collection.len() {
            if (i > start) && (collection[i] == collection[start]) {
                v(collection, i, start, State::Repeated);
                continue;
            }
            v(collection, i, start, State::Selected);
            collection.swap(start, i);
            v(collection, i, start, State::Swapped);
            perm(collection, start + 1, v);
            collection.swap(start, i);
        }
    }
}

pub fn run<F>(v: &mut F)
where
    F: FnMut(&mut Vec<char>, usize, usize, State),
{
    // collection: &mut Vec<char>,
    //read collection from "input.txt"
    //file content:
    // n//number of elements
    // elements

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
    collection.sort();
    perm(&mut collection, 0, v);
}
