pub mod models;

use {
    indexmap::IndexMap,
    rand::{random_bool, random_range, seq::SliceRandom},
    std::{
        collections::BTreeMap,
        hash::{DefaultHasher, Hash, Hasher},
        sync::Arc,
    },
};

type Id = u64;

#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    pub name: Arc<str>,
    pub leader_name: Option<Arc<str>>,
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
        let name: Arc<str> = Arc::from(name);

        let leader_name: Option<Arc<str>> = leader_name.map(|name| Arc::from(name));

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

    pub event: Vec<Vec<usize>>,
    pub idx: Vec<Vec<usize>>,

    pub collisions: IndexMap<(usize, usize), usize>,

    pub len: usize,

    pub cost: Cost,
}

pub const LAMBDA_OPT_DEFAULT: f64 = 0.99;
pub const AGING_OPT_DEFAULT: usize = 10000;

impl Schedule {
    pub fn new(scheme: Vec<Vec<Event>>) -> Self {
        let lens = scheme
            .iter()
            .map(|i| i.iter().map(|e| e.len).sum::<usize>())
            .collect::<Vec<usize>>();
        let idx = scheme.iter().map(|line| vec![0; line.len() + 1]).collect();
        let mut me = Self {
            scheme,
            len: lens.iter().sum(),
            cost: 0,
            event: lens.into_iter().map(|len| vec![0; len]).collect(),
            collisions: IndexMap::new(),
            idx,
        };
        me.update();
        me
    }

    pub fn update(&mut self) {
        let mut counts: Vec<BTreeMap<Id, usize>> = vec![BTreeMap::new(); self.len];
        self.cost = 0;

        for line in 0..self.scheme.len() {
            let mut j = 0;
            for i in 0..self.scheme[line].len() {
                let event = &self.scheme[line][i];
                self.idx[line][i + 1] = self.idx[line][i] + event.len;
                for _ in 0..event.len {
                    if let Some(id) = event.leader_id {
                        let prev_count = *counts[j].get(&id).unwrap_or(&0);
                        self.cost += prev_count as Cost;
                        counts[j].insert(id, prev_count + 1);
                    }

                    self.event[line][j] = i;
                    j += 1;
                }
            }
        }

        self.collisions.clear();

        for line in 0..self.scheme.len() {
            let mut j = 0;
            for i in 0..self.scheme[line].len() {
                let event = &self.scheme[line][i];
                for _ in 0..event.len {
                    if let Some(id) = event.leader_id {
                        let c = *counts[j].get(&id).unwrap_or(&0);
                        if c >= 2 {
                            let prev = *self.collisions.get(&(line, i)).unwrap_or(&0);
                            self.collisions.insert((line, i), prev + c - 1);
                        }
                    }
                    j += 1;
                }
            }
        }
    }

    fn swap(&mut self, line: usize, a: usize, b: usize) {
        if a == b {
            return;
        }
        if self.scheme[line][a].len == self.scheme[line][b].len {
            let ai = self.idx[line][a];
            let bi = self.idx[line][b];

            let mut new_cost: i64 = self.cost as i64;

            self.collisions.swap_remove(&(line, a));
            self.collisions.swap_remove(&(line, b));

            let mut coll_a = 0usize;
            let mut coll_b = 0usize;

            let len = self.scheme[line][a].len;
            for l in 0..self.scheme.len() {
                if l == line {
                    continue;
                }

                for i in ai..ai + len {
                    if i >= self.event[l].len() {
                        break;
                    }
                    let index = self.event[l][i];
                    let event = &self.scheme[l][index];
                    if event.leader_id == self.scheme[line][b].leader_id {
                        let prev = self.collisions.get(&(l, index)).unwrap_or(&0);
                        self.collisions.insert((l, index), prev + 1);
                        new_cost += 1;
                        coll_a += 1;
                    }
                    if event.leader_id == self.scheme[line][a].leader_id {
                        let prev = self
                            .collisions
                            .get(&(l, index))
                            .unwrap_or_else(|| panic!("{line} {a} {b} {index} {l}"));
                        if *prev > 1 {
                            self.collisions.insert((l, index), prev - 1);
                        } else {
                            self.collisions.swap_remove(&(l, index));
                        }
                        new_cost -= 1;
                    }
                }

                for i in bi..bi + len {
                    if i >= self.event[l].len() {
                        break;
                    }
                    let index = self.event[l][i];
                    let event = &self.scheme[l][index];
                    if event.leader_id == self.scheme[line][a].leader_id {
                        let prev = self.collisions.get(&(l, index)).unwrap_or(&0);
                        self.collisions.insert((l, index), prev + 1);
                        new_cost += 1;
                        coll_b += 1;
                    }
                    if event.leader_id == self.scheme[line][b].leader_id {
                        // dbg!((l, index));
                        let prev = self
                            .collisions
                            .get(&(l, index))
                            .unwrap_or_else(|| panic!("{line} {a} {b} {index} {l}"));

                        if *prev > 1 {
                            self.collisions.insert((l, index), prev - 1);
                        } else {
                            self.collisions.swap_remove(&(l, index));
                        }
                        new_cost -= 1;
                    }
                }
            }

            if coll_a >= 1 {
                self.collisions.insert((line, a), coll_a);
            }

            if coll_b >= 1 {
                self.collisions.insert((line, b), coll_b);
            }

            self.scheme[line].swap(a, b);
            self.cost = new_cost as Cost;
        } else {
            self.scheme[line].swap(a, b);
            self.update();
        }
    }

    pub fn optimize<F>(
        &mut self,
        opt_lambda: f64,
        opt_aging: usize,
        shuffling: bool,
        greedly: bool,
        mut tick_func: F,
    ) where
        F: FnMut(),
    {
        let n = self.scheme.len();
        if n == 0 {
            return;
        }

        if shuffling {
            for i in &mut self.scheme {
                i.shuffle(&mut rand::rng());
            }
            self.update();
        }

        let mut t = 1f64;

        for _ in 0..opt_aging {
            t *= opt_lambda;

            let (i, a, b) = if greedly {
                let (i, a) = *self
                    .collisions
                    .get_index(random_range(0..self.collisions.len()))
                    .unwrap()
                    .0;
                (i, a, random_range(0..self.scheme[i].len()))
            } else {
                let i = random_range(0..self.scheme.len());
                (
                    i,
                    random_range(0..self.scheme[i].len()),
                    random_range(0..self.scheme[i].len()),
                )
            };
            let prev_cost = self.cost;
            self.swap(i, a, b);
            let new_cost = self.cost;
            if prev_cost < new_cost
                && !random_bool(f64::exp((prev_cost as i64 - new_cost as i64) as f64 / t))
            {
                self.swap(i, a, b);
            }
            tick_func();
            if self.cost == 0 {
                break;
            }
        }

        self.update();
    }
}
