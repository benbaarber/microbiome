use std::{collections::HashMap, sync::LazyLock};

use indexmap::IndexMap;
use rand::{Rng, RngExt, seq::IndexedRandom};
use rand_distr::{Normal, Uniform};

use crate::util::LogUniform;

const NUM_INPUTS: usize = 1; // placeholder
const NUM_OUTPUTS: usize = 1; // placeholder
const INITIAL_HIDDEN_COUNT: usize = 8;
const INITIAL_DENSITY: f32 = 3.0;

const HIDDEN_START_IX: usize = NUM_INPUTS + NUM_OUTPUTS;
const INITIAL_NUM_NEURONS: usize = NUM_INPUTS + NUM_OUTPUTS + INITIAL_HIDDEN_COUNT;

// -------------------- Evolution

pub struct InitDistributions {
    tau: LogUniform,
    bias: Uniform<f32>,
    weight: Uniform<f32>,
}

impl InitDistributions {
    pub fn init() -> Self {
        Self {
            weight: Uniform::new(-2.0, 2.0).unwrap(),
            bias: Uniform::new(-1.0, 1.0).unwrap(),
            tau: LogUniform::new(0.05, 1.0),
        }
    }
}

pub struct MutationDistributions {
    tau: Normal<f32>,
    bias: Normal<f32>,
    gain: Normal<f32>,
    weight: Normal<f32>,
}

impl MutationDistributions {
    pub fn init() -> Self {
        Self {
            weight: Normal::new(0.0, 0.3).unwrap(),
            bias: Normal::new(0.0, 0.2).unwrap(),
            tau: Normal::new(0.0, 0.15).unwrap(),
            gain: Normal::new(0.0, 0.15).unwrap(),
        }
    }
}

static INIT_DISTRS: LazyLock<InitDistributions> = LazyLock::new(InitDistributions::init);
static MUT_DISTRS: LazyLock<MutationDistributions> = LazyLock::new(MutationDistributions::init);
const MUT_TAU_RATE: f64 = 0.05;
const MUT_BIAS_RATE: f64 = 0.1;
const MUT_GAIN_RATE: f64 = 0.05;
const MUT_WEIGHT_RATE: f64 = 0.1;
const ADD_SYNAPSE_RATE: f64 = 0.06;
const RM_SYNAPSE_RATE: f64 = 0.04;
const ADD_NEURON_RATE: f64 = 0.03;
const RM_NEURON_RATE: f64 = 0.02;

#[derive(Clone, Copy, Debug)]
pub struct NeuronGene {
    id: u32,
    tau: f32,
    bias: f32,
    gain: f32,
}

impl NeuronGene {
    pub fn new(id: u32, rng: &mut impl Rng) -> Self {
        Self {
            id,
            tau: rng.sample(INIT_DISTRS.tau),
            bias: rng.sample(INIT_DISTRS.bias),
            gain: 1.0,
        }
    }
}

pub struct CTRNNGenome {
    neurons: Vec<NeuronGene>,
    synapses: IndexMap<(u32, u32), f32>,
    next_neuron_id: u32,
}

impl CTRNNGenome {
    pub fn new(rng: &mut impl Rng) -> Self {
        let neurons = (0..INITIAL_NUM_NEURONS)
            .map(|i| NeuronGene::new(i as u32, rng))
            .collect::<Vec<_>>();

        let mut synapses = IndexMap::new();
        let synapse_chance = INITIAL_DENSITY as f64 / INITIAL_NUM_NEURONS as f64;
        for n1 in &neurons {
            for n2 in &neurons {
                if rng.random_bool(synapse_chance) {
                    synapses.insert((n1.id, n2.id), rng.sample(INIT_DISTRS.weight));
                }
            }
        }

        Self {
            neurons,
            synapses,
            next_neuron_id: INITIAL_NUM_NEURONS as u32,
        }
    }

    /// Average number of synapses per neuron
    pub fn density(&self) -> f32 {
        self.synapses.len() as f32 / self.neurons.len() as f32
    }

    pub fn num_hidden_neurons(&self) -> u32 {
        (self.neurons.len() as i32 - (NUM_INPUTS + NUM_OUTPUTS) as i32).max(0) as u32
    }

    fn try_add_synapse(&mut self, src: u32, tgt: u32, rng: &mut impl Rng) {
        if self.synapses.contains_key(&(src, tgt)) {
            self.synapses
                .insert((src, tgt), rng.sample(INIT_DISTRS.weight));
        }
    }

    pub fn mutate(&mut self, rng: &mut impl Rng) {
        for n in &mut self.neurons {
            if rng.random_bool(MUT_TAU_RATE) {
                n.tau *= rng.sample(MUT_DISTRS.tau).exp();
            }
            if rng.random_bool(MUT_BIAS_RATE) {
                n.bias += rng.sample(MUT_DISTRS.bias);
            }
            if rng.random_bool(MUT_GAIN_RATE) {
                n.gain *= rng.sample(MUT_DISTRS.gain).exp();
            }
        }

        for w in self.synapses.values_mut() {
            if rng.random_bool(MUT_WEIGHT_RATE) {
                *w += rng.sample(MUT_DISTRS.weight);
            }
        }

        if rng.random_bool(ADD_SYNAPSE_RATE) && !self.neurons.is_empty() {
            let n1 = self.neurons.choose(rng).unwrap();
            let n2 = self.neurons.choose(rng).unwrap();
            self.try_add_synapse(n1.id, n2.id, rng);
        }

        if rng.random_bool(RM_SYNAPSE_RATE) && !self.synapses.is_empty() {
            let idx = rng.random_range(0..self.synapses.len());
            self.synapses.swap_remove_index(idx);
        }

        if rng.random_bool(ADD_NEURON_RATE) {
            let d = self.density();
            let n = NeuronGene::new(self.next_neuron_id, rng);
            self.next_neuron_id += 1;
            self.neurons.push(n);

            let half_syn_count = (d / 2.0).round().max(1.0) as u32;
            for _ in 0..half_syn_count {
                let other = self.neurons.choose(rng).unwrap();
                self.try_add_synapse(n.id, other.id, rng);
            }
            for _ in 0..half_syn_count {
                let other = self.neurons.choose(rng).unwrap();
                self.try_add_synapse(other.id, n.id, rng);
            }
        }

        if rng.random_bool(RM_NEURON_RATE) && self.num_hidden_neurons() > 0 {
            let idx = rng.random_range(HIDDEN_START_IX..self.neurons.len());
            let n = self.neurons.swap_remove(idx);
            self.synapses
                .retain(|(src, tgt), _| *src != n.id && *tgt != n.id);
        }
    }

    pub fn decode(&self) -> CTRNN {
        let num_neurons = self.neurons.len();
        let mut inv_taus = vec![0.0; num_neurons];
        let mut biases = vec![0.0; num_neurons];
        let mut gains = vec![0.0; num_neurons];
        let mut neuron_id_to_ix = HashMap::with_capacity(num_neurons);
        for (i, n) in self.neurons.iter().enumerate() {
            inv_taus[i] = 1.0 / n.tau;
            biases[i] = n.bias;
            gains[i] = n.gain;
            neuron_id_to_ix.insert(n.id, i);
        }

        let mut counts = vec![0u32; num_neurons]; // num inbound synapses per neuron
        for (_, t) in self.synapses.keys() {
            let tgt = neuron_id_to_ix[t];
            counts[tgt] += 1;
        }

        let mut synapse_offsets = vec![0usize; num_neurons + 1];
        for i in 0..num_neurons {
            synapse_offsets[i + 1] = synapse_offsets[i] + counts[i] as usize;
        }

        let num_enabled = synapse_offsets[num_neurons];
        let mut synapse_sources = vec![0usize; num_enabled];
        let mut synapse_weights = vec![0.0f32; num_enabled];
        let mut cursor = synapse_offsets.clone();
        for ((s, t), w) in &self.synapses {
            let src = neuron_id_to_ix[s];
            let tgt = neuron_id_to_ix[t];
            let ix = cursor[tgt];
            synapse_sources[ix] = src;
            synapse_weights[ix] = *w;
            cursor[tgt] += 1;
        }

        CTRNN::new(
            inv_taus,
            biases,
            gains,
            synapse_offsets,
            synapse_sources,
            synapse_weights,
        )
    }
}

// -------------------- Computation

const NUM_SUBSTEPS: usize = 8;
const DT: f32 = 1.0 / (60.0 * NUM_SUBSTEPS as f32);

fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + f32::exp(-x))
}

pub struct CTRNN {
    potentials: Vec<f32>,
    outputs: Vec<f32>, // cached from last euler step
    inv_taus: Vec<f32>,
    biases: Vec<f32>,
    gains: Vec<f32>,

    // CSR storage of synapses
    synapse_offsets: Vec<usize>, // len: potentials.len() + 1
    synapse_sources: Vec<usize>,
    synapse_weights: Vec<f32>,
}

impl CTRNN {
    pub fn new(
        inv_taus: Vec<f32>,
        biases: Vec<f32>,
        gains: Vec<f32>,
        synapse_offsets: Vec<usize>,
        synapse_sources: Vec<usize>,
        synapse_weights: Vec<f32>,
    ) -> Self {
        let mut net = Self {
            potentials: vec![0.0; inv_taus.len()],
            outputs: vec![0.0; inv_taus.len()],
            inv_taus,
            biases,
            gains,
            synapse_offsets,
            synapse_sources,
            synapse_weights,
        };

        net.compute_outputs();
        net
    }

    fn compute_outputs(&mut self) {
        for i in 0..self.potentials.len() {
            self.outputs[i] = sigmoid(self.gains[i] * (self.potentials[i] + self.biases[i]));
        }
    }

    pub fn step(&mut self, input: &[f32]) {
        for _ in 0..NUM_SUBSTEPS {
            let n = self.potentials.len();

            for i in 0..n {
                let start = self.synapse_offsets[i];
                let end = self.synapse_offsets[i + 1];
                let mut total_input = 0.0;
                for k in start..end {
                    total_input += self.synapse_weights[k] * self.outputs[self.synapse_sources[k]];
                }
                if i < NUM_INPUTS {
                    total_input += input[i];
                }

                let dy = (total_input - self.potentials[i]) * self.inv_taus[i];
                self.potentials[i] = self.potentials[i] + DT * dy;
            }

            self.compute_outputs();
        }
    }

    pub fn read(&self) -> &[f32] {
        &self.outputs[NUM_INPUTS..NUM_INPUTS + NUM_OUTPUTS]
    }
}
