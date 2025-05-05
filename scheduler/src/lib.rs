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
    pub len: usize,

    pub cost: Cost,
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
        let mut me = Self {
            scheme,
            len,
            cost: 0,
        };
        me.update_cost();
        me
    }

    pub fn update_cost(&mut self) {
        let mut counts: Vec<BTreeMap<Id, usize>> = vec![BTreeMap::new(); self.len];
        self.cost = 0;
        self.scheme.iter().for_each(|i| {
            i.iter().fold(0, |mut j, event| {
                for _ in 0..event.len {
                    if let Some(leader_id) = event.leader_id {
                        let prev_count = counts[j].get(&leader_id).unwrap_or(&0).clone();
                        self.cost += prev_count as Cost;
                        counts[j].insert(leader_id, prev_count + 1);
                        j += 1;
                    }
                }
                j
            });
        });
    }

    pub fn optimize<F>(
        &mut self,
        opt_lambda: f64,
        opt_aging: usize,
        shuffling: bool,
        greedily: bool,
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
            self.update_cost();
        }

        if greedily {
            let mut columns_sets: Vec<BTreeSet<Id>> = vec![BTreeSet::new(); self.len];
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
        let mut t = 1f64;

        for _ in 0..opt_aging {
            t *= opt_lambda;

            let i = random_range(0..self.scheme.len());
            let a = random_range(0..self.scheme[i].len());
            let b = random_range(0..self.scheme[i].len());

            let prev_cost = self.cost;
            self.scheme[i].swap(a, b);
            self.update_cost();
            let new_cost = self.cost;

            if prev_cost < new_cost
                && !random_bool(f64::exp((prev_cost as i64 - new_cost as i64) as f64 / t))
            {
                self.scheme[i].swap(a, b);
                self.cost = prev_cost;
            }
            tick_func();
        }
    }
}
