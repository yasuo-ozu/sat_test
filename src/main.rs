#[macro_use]
extern crate anyhow;
extern crate hsl;
extern crate rand;
extern crate screwsat;
use anyhow::Result;
use plotlib::page::Page;
use plotlib::repr::Plot;
use plotlib::style::PointStyle;
use plotlib::view::{ContinuousView, View};
use rand::rngs::SmallRng;
use rand::{Rng, RngCore, SeedableRng};
use screwsat::{solver, util};
use solver::Status;
use solver::{Lit, LitBool, Var};
use std::time::Instant;
const NUM_PLOTS: usize = 8192;

fn random_state<R: RngCore>(r: &mut R, n: usize) -> Vec<bool> {
	const bits_of_u32: usize = std::mem::size_of::<u32>() * 8;
	let mut ret = Vec::new();
	loop {
		let mut value = r.next_u32();
		for _ in 0..bits_of_u32 {
			if ret.len() == n {
				return ret;
			}
			ret.push((value & 1) == 1);
			value >>= 1;
		}
	}
}

fn distance(start: &[bool], end: &[LitBool]) -> usize {
	assert_eq!(start.len(), end.len());
	start
		.iter()
		.zip(end.iter())
		.filter(|(a, b)| !**a && **b == LitBool::True || **a && **b == LitBool::False)
		.count()
}

fn color_from_val(val: usize, max: usize) -> String {
	let (r, g, b) = hsl::HSL {
		h: (val as f64 * 360.0) / max as f64,
		s: 1.0,
		l: 0.5,
	}
	.to_rgb();
	format!("#{:02X}{:02X}{:02X}", r, g, b)
}

fn main() -> Result<()> {
	if let [_, infile, outfile] = std::env::args().collect::<Vec<String>>().as_slice() {
		let input = std::fs::File::open(infile)?;
		let cnf = util::parse_cnf(input)?;
		let n = cnf.var_num.unwrap();
		let mut rng = SmallRng::from_entropy();
		let mut data = vec![Vec::new(); n];
		let mut state = random_state(&mut rng, n);
		let mut next_step_dist: Option<usize> = None;
		for _ in 0..NUM_PLOTS {
			let mut solver = solver::Solver::new(n, &cnf.clauses);
			for (i, b) in state.iter().enumerate() {
				solver.set_polarity_and_priority(Var(i as u32), *b, 0);
			}
			let t_start = Instant::now();
			if solver.solve(None) != Status::Sat {
				bail!("UNSAT");
			}
			let time = t_start.elapsed().as_micros();
			let dist = distance(&state, &solver.models);
			state = solver.models.iter().map(|b| b == &LitBool::True).collect();
			// let nsd = if let Some(nsd) = next_step_dist {
			// 	data[nsd].push((dist as f64, time as f64));
			// 	if dist == 0 {
			// 		1
			// 	} else {
			// 		if nsd + 1 < n {
			// 			nsd + 1
			// 		} else {
			// 			1
			// 		}
			// 	}
			// } else {
			// 	1
			// };
			// next_step_dist = Some(nsd);
			if let Some(nsd) = next_step_dist {
				data[nsd].push((dist as f64, time as f64));
			}
			let nsd = rng.gen_range(0..n);
			next_step_dist = Some(nsd);
			let mut v = (0..n)
				.map(|i| (rng.next_u32(), i < nsd))
				.collect::<Vec<_>>();
			v.sort_by_key(|(a, _)| *a);

			for (idx, (_, b)) in v.iter().enumerate() {
				if *b {
					state[idx] = !state[idx];
				}
			}
		}
		let v = (0..n).rev().fold(ContinuousView::new(), |view, dist| {
			view.add(
				Plot::new(data[dist].clone())
					.point_style(PointStyle::new().colour(color_from_val(dist, n)).size(2.0)),
			)
		});
		let v = v
			.x_range(0., n as f64)
			.x_label("manhattan distance")
			.y_label("time [us]");
		Page::single(&v).save(outfile).unwrap();
		Ok(())
	} else {
		bail!("usage: sat_test [cnffile] [svgfile]")
	}
}
