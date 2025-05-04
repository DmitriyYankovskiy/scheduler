use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    rc::Rc,
};

use rand::{random_bool, random_range, rng, seq::SliceRandom};

type Id = u64;

#[derive(Debug, Clone)]
pub struct Event {
    pub name: Rc<str>,
    pub leader_name: Option<Rc<str>>,
    pub leader_id: Option<Id>,
    pub len: usize,
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

pub type Cost = u64;

pub struct Schedule {
    pub scheme: Vec<Vec<Event>>,
    pub len: usize,
    pub opt_lambda: f64,
    pub opt_aging: usize,
}

impl Schedule {
    pub fn new(scheme: Vec<Vec<Event>>) -> Self {
        let len = scheme
            .iter()
            .map(|i| i.iter().map(|e| e.len).sum::<usize>())
            .max()
            .unwrap_or(0);
        Self {
            scheme,
            len,
            opt_lambda: 0.99,
            opt_aging: 10000,
        }
    }

    pub fn cost(&self) -> Cost {
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
    }

    pub fn optimize(&mut self) {
        let n = self.scheme.len();
        if n == 0 {
            return;
        }

        for i in &mut self.scheme {
            i.shuffle(&mut rand::rng());
        }

        let m = self.scheme.iter().map(|i| i.len()).max().unwrap();
        let mut t = 1f64;

        for _ in 0..self.opt_aging {
            t *= self.opt_lambda;

            let i = random_range(0..self.scheme.len());
            let a = random_range(0..self.scheme[i].len());
            let b = random_range(0..self.scheme[i].len());

            let cost = self.cost();
            self.scheme[i].swap(a, b);
            let new_cost = self.cost();

            if cost < new_cost && !random_bool(f64::exp((cost as i64 - new_cost as i64) as f64 / t))
            {
                self.scheme[i].swap(a, b);
            }
        }
    }
}
