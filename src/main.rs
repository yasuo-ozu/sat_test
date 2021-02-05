#[macro_use]
extern crate anyhow;
extern crate rand;
extern crate screwsat;
use anyhow::Result;
use rand::rngs::SmallRng;
use rand::{RngCore, SeedableRng};
use screwsat::{solver, util};
use solver::Status;
use solver::{Lit, LitBool, Var};
use std::time::Instant;

fn main() -> Result<()> {
    if let [_, infile] = std::env::args().collect::<Vec<String>>().as_slice() {
        let input = std::fs::File::open(infile)?;
        let cnf = util::parse_cnf(input)?;
        let n = cnf.var_num.unwrap();
        let mut rng = SmallRng::from_entropy();
        let clauses = cnf.clauses;
        let mut clauses_incremental = clauses.clone();
        let max_increment = 50;
        let max_shuffle = 10;
        let max_iter = 1;
        let mut res = vec![Vec::new(); n];
        for _ in 0..max_increment {
            let mut solver = solver::Solver::new(n, &clauses_incremental);
            for i in 0..n {
                solver.set_polarity_and_priority(Var(i as u32), (rng.next_u32() & 1) > 0, 0);
            }
            if solver.solve(None) != Status::Sat {
                break;
            }
            let mut ans = solver
                .models
                .iter()
                .map(|b| match b {
                    LitBool::True => true,
                    LitBool::False => false,
                    _ => panic!(),
                })
                .collect::<Vec<_>>();
            let clause = ans
                .iter()
                .enumerate()
                .map(|(i, b)| Lit::new(i as u32, !*b))
                .collect::<Vec<_>>();
            clauses_incremental.push(clause);

            let mut res_inner = vec![0; n];
            for _ in 0..max_shuffle {
                let mut indexes = (0..n)
                    .map(|i| (i, rng.next_u32() as usize))
                    .collect::<Vec<_>>();
                indexes.sort_by_key(|(_, b)| *b);
                for (distance, (i, _)) in indexes.iter().enumerate() {
                    let mut sum = 0;
                    for _ in 0..max_iter {
                        let mut solver = solver::Solver::new(n, &clauses);
                        for i in 0..n {
                            solver.set_polarity_and_priority(Var(i as u32), ans[i], 0);
                        }
                        let t_start = Instant::now();
                        solver.solve(None);
                        sum += t_start.elapsed().as_micros();
                    }
                    res_inner[distance] += sum / max_iter;
                    ans[*i] = !ans[*i];
                }
            }
            for (i, v) in res_inner.iter().enumerate() {
                res[i].push(v / max_shuffle);
            }
        }
        for (i, v) in res.iter().enumerate() {
            println!("{} {:?}", i, v);
        }
        Ok(())
    } else {
        bail!("usage: sat_test [cnffile]")
    }
}
