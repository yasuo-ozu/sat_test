#[macro_use]
extern crate anyhow;
extern crate rustqubo;
extern crate screwsat;
extern crate spoon_q;
use anyhow::Result;
use plotlib::page::Page;
use plotlib::repr::Plot;
use plotlib::style::PointStyle;
use plotlib::view::{ContinuousView, View};
use rustqubo::Expr;
use screwsat::solver::{Lit, LitBool, Status, Var};
use spoon_q::file::File;
use std::time::Instant;

fn distance(start: &[bool], end: &[LitBool]) -> usize {
	assert_eq!(start.len(), end.len());
	start
		.iter()
		.zip(end.iter())
		.filter(|(a, b)| !**a && **b == LitBool::True || **a && **b == LitBool::False)
		.count()
}

const NUM_PLOTS: usize = 10;
fn main() -> Result<()> {
	if let [_, infile, outfile] = std::env::args().collect::<Vec<String>>().as_slice() {
		let cnf = {
			let input = std::fs::File::open(infile)?;
			screwsat::util::parse_cnf(input)?
		};
		let n = cnf.var_num.unwrap();
		let file = spoon_q::file::File::from_file(infile).unwrap();
		let mut cxt = spoon_q::context::Context::new(file);
		let mut tknzr = spoon_q::token::Tokenizer::new(&mut cxt);
		let cond = spoon_q::dimacs::DimacsGenerator::generate_cond(&mut tknzr)
			.unwrap()
			.unwrap()
			.collect_andcond();
		let mut hmlt = cond
			.iter()
			.enumerate()
			.map(|(i, c)| Expr::Constraint {
				label: format!("Constraint {:}", i),
				expr: Box::new(
					spoon_q::optim::Optim::from_cond(c)
						.get_strategy(&spoon_q::optim::OptimStrategy::Optimize)
						.unwrap(),
				),
			})
			.fold(spoon_q::optim::OptimExpr::Number(0.0), |e1, e2| e1 + e2);
		let compiled = hmlt.compile();
		let mut solver = rustqubo::solve::SimpleSolver::new(&compiled);
		solver.iterations = 1;
		solver.generations = 1;
		solver.samples = 1;
		solver.solver_generator.sweeps_per_round = 1;
		let mut data = Vec::with_capacity(NUM_PLOTS);
		for _ in 0..NUM_PLOTS {
			let (energy, sol, _constraints) = solver.solve_with_constraints().unwrap();
			let mut satsolv = screwsat::solver::Solver::new(n, &cnf.clauses);
			let mut before = vec![false; n];
			for key in sol.keys() {
				let b = sol[key];
				if let spoon_q::optim::Qubit::Val(i) = key {
					before[*i] = b;
					satsolv.set_polarity_and_priority(Var(*i as u32), b, 0);
				}
			}
			if satsolv.solve(None) != Status::Sat {
				bail!("UNSAT");
			}
			let dist = distance(&before, &satsolv.models);
			data.push((dist as f64, energy as f64));
		}
		let v = ContinuousView::new()
			.add(Plot::new(data).point_style(PointStyle::new().colour("green").size(2.0)))
			.x_range(0., n as f64)
			.x_label("manhattan distance")
			.y_label("remaining energy");
		Page::single(&v).save(outfile).unwrap();
		Ok(())
	} else {
		bail!("usage: sat_test [cnffile] [svgfile]")
	}
}
