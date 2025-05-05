use std::{
    collections::{HashMap, HashSet},
    hash::{DefaultHasher, Hash, Hasher},
    io::{self, Write},
    rc::Rc,
};

use rand::{random_bool, random_range, seq::SliceRandom};

type Id = u64;

#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    pub name: Rc<str>,
    pub leader_name: Option<Rc<str>>,
    pub leader_id: Option<Id>,
    pub len: usize,
}

impl Hash for Event {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.leader_id.hash(state);
    }
}

impl Event {
    pub fn new(name: Box<str>, leader_name: Option<Box<str>>, len: usize) -> Self {
        let name: Rc<str> = Rc::from(name);

        let leader_name: Option<Rc<str>> = leader_name.map(|name| Rc::from(name));

        let leader_id = leader_name.clone().map(|name| {
            let mut hasher = DefaultHasher::new();
            name.hash(&mut hasher);
            hasher.finish()
        });

        Self {
            name,
            leader_name,
            leader_id,
            len,
        }
    }
}

pub enum Precalc {
    Shuffling,
    Greedily,
}

pub type Cost = u64;

pub struct Schedule {
    pub scheme: Vec<Vec<Event>>,
    pub len: usize,
    pub cost: Cost,

    pub len_pref_sums: Vec<Vec<usize>>,
}

pub const LAMBDA_OPT_DEFAULT: f64 = 0.99;
pub const AGING_OPT_DEFAULT: usize = 10000;

impl Schedule {
    pub fn new(scheme: Vec<Vec<Event>>) -> Self {
        let len = scheme
            .iter()
            .map(|i| i.iter().map(|e| e.len).sum::<usize>())
            .max()
            .unwrap_or(0);

        let len_pref_sums = scheme
            .iter()
            .map(|line| {
                let mut pref = vec![0; line.len() + 1];
                for i in 0..line.len() {
                    pref[i + 1] = pref[i] + line[i].len;
                }
                pref
            })
            .collect();

        let mut me = Self {
            scheme,
            len,
            cost: 0,
            len_pref_sums,
        };
        me.update_cost();
        me
    }

    fn update_cost(&mut self) {
        self.cost = {
            let mut counts: Vec<HashMap<Id, usize>> = vec![HashMap::new(); self.len];

            for i in &self.scheme {
                let mut j = 0;
                for event in i {
                    for _ in 0..event.len {
                        if let Some(leader_id) = event.leader_id {
                            let prev_count = counts[j].get(&leader_id).unwrap_or(&0).clone();
                            counts[j].insert(leader_id, prev_count + 1);
                            j += 1;
                        }
                    }
                }
            }

            counts
                .into_iter()
                .map(|i| {
                    i.into_iter()
                        .map(|(_, j)| j as u64 * (j - 1) as u64 / 2)
                        .sum::<u64>()
                })
                .sum::<u64>()
        };
    }

    fn swap(&mut self, line: usize, idx: usize) {
        let l_id = self.scheme[line][idx].leader_id;
        let r_id = self.scheme[line][idx + 1].leader_id;

        let begin = self.len_pref_sums[line][idx];
        let end = self.len_pref_sums[line][idx + 2];

        let divider = self.scheme[line][idx].len + begin;
        let new_divider = self.scheme[line][idx + 1].len + begin;

        fn cross_len(l1: usize, r1: usize, l2: usize, r2: usize) -> usize {
            let l = std::cmp::max(l1, l2);
            let r = std::cmp::min(r1, r2);
            if r <= l { 0 } else { r - l }
        }

        fn identity<T>(x: T) -> T {
            x
        }

        for i in 0..self.scheme.len() {
            if i == line {
                continue;
            }

            let begin_idx = self.len_pref_sums[i]
                .binary_search(&(begin + 1))
                .unwrap_or_else(identity)
                - 1;
            let end_idx = self.len_pref_sums[i]
                .binary_search(&end)
                .unwrap_or_else(identity);

            for j in begin_idx..end_idx {
                if self.scheme[i][j].leader_id == l_id {
                    self.cost -= cross_len(
                        begin,
                        divider,
                        self.len_pref_sums[i][j],
                        self.len_pref_sums[i][j + 1],
                    ) as u64;

                    self.cost += cross_len(
                        new_divider,
                        end,
                        self.len_pref_sums[i][j],
                        self.len_pref_sums[i][j + 1],
                    ) as u64;
                }

                if self.scheme[i][j].leader_id == r_id {
                    self.cost -= cross_len(
                        divider,
                        end,
                        self.len_pref_sums[i][j],
                        self.len_pref_sums[i][j + 1],
                    ) as u64;

                    self.cost += cross_len(
                        begin,
                        new_divider,
                        self.len_pref_sums[i][j],
                        self.len_pref_sums[i][j + 1],
                    ) as u64;
                }
            }
        }

        self.scheme[line].swap(idx, idx + 1);
    }

    pub fn optimize(&mut self, opt_lambda: f64, opt_aging: usize, shuffling: bool, greedily: bool) {
        let n = self.scheme.len();
        if n == 0 {
            return;
        }

        if shuffling {
            for i in &mut self.scheme {
                i.shuffle(&mut rand::rng());
            }
        }

        if greedily {
            let mut columns_sets: Vec<HashSet<Id>> = vec![HashSet::new(); self.len];
            for i in 0..n {
                let line = &mut self.scheme[i];
                let mut free_elements = line.clone();
                line.clear();
                let mut time = 0;
                for j in 0..free_elements.len() {
                    let mut use_idx = None;
                    for e_i in 0..free_elements.len() {
                        let e = &free_elements[e_i];
                        if e.leader_id == None
                            || !columns_sets[time].contains(&e.leader_id.unwrap())
                        {
                            if let Some(id) = e.leader_id {
                                for addi in 0..e.len {
                                    columns_sets[j + addi].insert(id);
                                }
                            }
                            time += e.len;
                            use_idx = Some(e_i);
                            break;
                        }
                    }

                    if use_idx == None {
                        let e = &free_elements[free_elements.len() - 1];
                        if let Some(id) = e.leader_id {
                            for addi in 0..e.len {
                                columns_sets[j + addi].insert(id);
                            }
                        }
                        time += e.len;
                        use_idx = Some(free_elements.len() - 1);
                    }

                    line.push(free_elements.remove(use_idx.unwrap()));
                }
            }
        }

        self.update_cost();
        let mut t = 1f64;

        if opt_aging > 100 {
            for _ in 0..opt_aging / (opt_aging / 100) {
                print!("█");
            }
            println!();
        }
        for iteration in 0..opt_aging {
            if opt_aging > 100 {
                if iteration % (opt_aging / 100) == 0 {
                    print!("█");
                    io::stdout().flush().unwrap();
                }
            }
            t *= opt_lambda;

            let i = random_range(0..self.scheme.len());
            let j = random_range(0..self.scheme[i].len() - 1);
            // let b = random_range(0..self.scheme[i].len());

            let prev_cost = self.cost;
            self.swap(i, j);
            let new_cost = self.cost;

            if prev_cost < new_cost
                && !random_bool(f64::exp((prev_cost as i64 - new_cost as i64) as f64 / t))
            {
                self.swap(i, j);
            }
        }
        println!();
    }
}
