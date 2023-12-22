use std::marker::PhantomData;

use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    halo2curves::FieldExt,
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Fixed, Selector},
    poly::Rotation,
};

use crate::gadgets::is_equal::IsEqualChip;

#[derive(Clone)]
// it is standard practice to define everything where numbers are in a generic prime field `F` (`FieldExt` are the traits of a prime field)
pub struct ShipCircuitConfig<F: FieldExt> {
    _marker: PhantomData<F>,
    ship: Column<Advice>,
    coords: Column<Advice>,
    count: Column<Advice>,
    is_equal: IsEqualChip<F>,
    s: Selector,
    #[allow(dead_code)]
    constant: Column<Fixed>,
}

impl<F: FieldExt> ShipCircuitConfig<F> {
    pub fn configure(meta: &mut ConstraintSystem<F>) -> Self {
        let [ship, coords, count] = [(); 3].map(|_| meta.advice_column());

        let constant = meta.fixed_column();

        let s = meta.selector();

        let is_equal = IsEqualChip::configure(
            meta,
            |meta| meta.query_advice(ship, Rotation::cur()),
            |meta| meta.query_advice(coords, Rotation::cur()),
        );

        // enable equality constraints
        [ship, coords, count].map(|column| meta.enable_equality(column));

        // enable constant
        meta.enable_constant(constant);

        meta.create_gate("count", |meta| {
            let count_curr = meta.query_advice(count, Rotation::cur());
            let count_next = meta.query_advice(count, Rotation::next());

            let s = meta.query_selector(s);

            // count_next = count + is_equal
            vec![
                // count_next = count_curr + is_eq
                s.clone() * (count_next - (count_curr + is_equal.expr())),
            ]
        });

        ShipCircuitConfig { ship, coords, count, is_equal, s, constant, _marker: PhantomData }
    }

    // Config is essentially synonymous with Chip, so we want to build some functionality into this Chip if we want
}

// we use the config to make a circuit:
#[derive(Clone, Default)]
pub struct ShipCircuit<F: FieldExt> {
    pub length: usize,
    pub ship: u64,
    pub coords: Vec<u64>,
    pub count: u64,
    _marker: PhantomData<F>,
}

impl<F: FieldExt> Circuit<F> for ShipCircuit<F> {
    type Config = ShipCircuitConfig<F>;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        ShipCircuitConfig::configure(meta)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "",
            |mut region| {
                region.name_column(|| "count", config.count);
                region.name_column(|| "ship", config.ship);
                region.name_column(|| "coords", config.coords);

                // initialize count to 0
                let mut count = region.assign_advice(
                    || "count",
                    config.count,
                    0,
                    || Value::known(F::from(self.count)),
                )?;
                // constrain the initial value of count to be 0
                region.constrain_constant(count.cell(), F::from(0))?;

                // ship coordinate
                let s_val = Value::known(F::from(self.ship));

                for row in 0..self.length {
                    // enable selectors
                    config.s.enable(&mut region, row)?;

                    // coordinate being compared against
                    let c_val = Value::known(F::from(self.coords.get(row).unwrap().clone()));

                    // assign s to every row in the ship column
                    region.assign_advice(|| "ship coordinate", config.ship, row, || s_val)?;

                    // assign c to the current row in the coords column
                    region.assign_advice(|| "compared coordinate", config.coords, row, || c_val)?;

                    let eq_val = config.is_equal.assign(&mut region, row, s_val, c_val)?;

                    count = region.assign_advice(
                        || "count",
                        config.count,
                        row + 1,
                        || eq_val + count.value().copied(),
                    )?;
                }
                Ok(())
            },
        )
    }
}

pub fn generate_circuit<F: FieldExt>(n: u32) -> ShipCircuit<F> {
    let ship = 0;
    let mut coords = Vec::new();
    for _ in 0..n {
        coords.push(0);
    }
    let count = 0;
    let circuit =
        ShipCircuit { length: n as usize, ship: ship, coords, count, _marker: PhantomData };
    circuit
}

#[cfg(test)]
mod test {
    use halo2_proofs::{
        // arithmetic::Field,
        // circuit::Value,
        dev::{MockProver, SimpleCircuitCost},
        halo2curves::pasta::{Eq, Fp},
    };
    // use rand::rngs::OsRng;

    use super::{generate_circuit, ShipCircuit};

    #[test]
    fn test_ship() {
        let k = 14;
        let n = 10;
        let circuit = generate_circuit::<Fp>(n);
        let x = SimpleCircuitCost::<Eq, ShipCircuit<Fp>>::measure(k, &circuit.clone());
        //     /// Power-of-2 bound on the number of rows in the circuit.
        // pub k: u32,
        // /// Maximum degree of the circuit.
        // pub max_deg: usize,
        // /// Number of advice columns.
        // pub advice_columns: usize,
        // /// Number of direct queries for instance columns.
        // pub instance_queries: usize,
        // /// Number of direct queries for advice columns.
        // pub advice_queries: usize,
        // /// Number of direct queries for fixed columns.
        // pub fixed_queries: usize,
        // /// Number of lookup arguments.
        // pub lookups: usize,
        // /// Number of columns in the global permutation.
        // pub permutation_cols: usize,
        // /// Number of distinct sets of points in the multiopening argument.
        // pub point_sets: usize,
        // /// Maximum rows used over all columns
        // pub max_rows: usize,
        // /// Maximum rows used over all advice columns
        // pub max_advice_rows: usize,
        // /// Maximum rows used over all fixed columns
        // pub max_fixed_rows: usize,
        // /// Number of fixed columns
        // pub num_fixed_columns: usize,
        // /// Number of advice columns
        // pub num_advice_columns: usize,
        // /// Number of instance columns
        // pub num_instance_columns: usize,
        // /// Total number of columns
        // pub num_total_columns: usize,
        // println!("k: {:?}", x.k);
        // println!("max_deg: {:?}", x.max_deg);
        // println!("advice_columns: {:?}", x.advice_columns);
        // println!("instance_queries: {:?}", x.instance_queries);
        // println!("advice_queries: {:?}", x.advice_queries);
        // println!("fixed_queries: {:?}", x.fixed_queries);
        // println!("lookups: {:?}", x.lookups);
        // println!("permutation_cols: {:?}", x.permutation_cols);
        // println!("point_sets: {:?}", x.point_sets);
        // println!("max_rows: {:?}", x.max_rows);
        println!("max_advice_rows: {:?}", x.max_advice_rows);
        println!("max_fixed_rows: {:?}", x.max_fixed_rows);
        println!("num_fixed_columns: {:?}", x.num_fixed_columns);
        println!("num_advice_columns: {:?}", x.num_advice_columns);
        // println!("num_instance_columns: {:?}", x.num_instance_columns);
        // println!("num_total_columns: {:?}", x.num_total_columns);
        let prover = MockProver::run(k, &circuit, vec![]).unwrap();
        prover.assert_satisfied();
    }
}
