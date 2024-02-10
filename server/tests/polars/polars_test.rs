use polars::prelude::*;

#[test]
fn main() {
	let lf1 = LazyFrame::scan_parquet("myfile_1.parquet", Default::default())?
		.group_by([col("ham")])
		.agg([
			// expressions can be combined into powerful aggregations
			col("foo")
				.sort_by([col("ham").rank(Default::default(), None)], [false])
				.last()
				.alias("last_foo_ranked_by_ham"),
			// every expression runs in parallel
			col("foo").cum_min(false).alias("cumulative_min_per_group"),
			// every expression runs in parallel
			col("foo").reverse().implode().alias("reverse_group"),
		]);

	let lf2 = LazyFrame::scan_parquet("myfile_2.parquet", Default::default())?
		.select([col("ham"), col("spam")]);

	let df = lf1
		.join(
			lf2,
			[col("reverse")],
			[col("foo")],
			JoinArgs::new(JoinType::Left),
		)
		// now we finally materialize the result.
		.collect()?;
}
