pub mod models;

use std::{
    collections::{BTreeMap, BTreeSet},
    hash::{DefaultHasher, Hash, Hasher},
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

    pub event: Vec<Vec<usize>>,
    pub idx: Vec<Vec<usize>>,

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
                        let prev_count = counts[j].get(&id).unwrap_or(&0).clone();
                        self.cost += prev_count as Cost;
                        counts[j].insert(id, prev_count + 1);
                    }

                    self.event[line][j] = i;
                    j += 1;
                }
            }
        }

        // self.scheme.iter().for_each(|i| {
        //     i.iter().fold(0, |mut j, event| {
        //         for _ in 0..event.len {
        //             if let Some(leader_id) = event.leader_id {
        //                 let prev_count = counts[j].get(&leader_id).unwrap_or(&0).clone();
        //                 self.cost += prev_count as Cost;
        //                 counts[j].insert(leader_id, prev_count + 1);
        //                 j += 1;
        //             }
        //         }
        //         j
        //     });
        // });
    }

    fn swap(&mut self, line: usize, a: usize, b: usize) {
        if self.scheme[line][a].len == self.scheme[line][b].len {
            let ai = self.idx[line][a];
            let bi = self.idx[line][b];

            let mut new_cost: i64 = self.cost as i64;

            let len = self.scheme[line][a].len;
            for l in 0..self.scheme.len() {
                if l == line {
                    continue;
                }

                for i in ai..ai + len {
                    if i >= self.event[l].len() {
                        break;
                    }
                    let event = &self.scheme[l][self.event[l][i]];
                    if event.leader_id == self.scheme[line][b].leader_id {
                        new_cost += 1;
                    }
                    if event.leader_id == self.scheme[line][a].leader_id {
                        new_cost -= 1;
                    }
                }

                for i in bi..bi + len {
                    if i >= self.event[l].len() {
                        break;
                    }
                    let event = &self.scheme[l][self.event[l][i]];
                    if event.leader_id == self.scheme[line][a].leader_id {
                        new_cost += 1;
                    }
                    if event.leader_id == self.scheme[line][b].leader_id {
                        new_cost -= 1;
                    }
                }
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

            let i = random_range(0..self.scheme.len());
            let a = random_range(0..self.scheme[i].len());
            let b = random_range(0..self.scheme[i].len());

            let prev_cost = self.cost;
            self.swap(i, a, b);
            let new_cost = self.cost;
            if prev_cost < new_cost
                && !random_bool(f64::exp((prev_cost as i64 - new_cost as i64) as f64 / t))
            {
                self.scheme[i].swap(a, b);
                self.cost = prev_cost;
            }
            tick_func();
            if self.cost == 0 {
                break;
            }
        }

        self.update();
    }
}
