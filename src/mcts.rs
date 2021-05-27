use crate::env::{Env, BLACK, WHITE};
use crate::rand::rngs::StdRng;
use crate::rand::SeedableRng;
use std::time::Instant;

pub struct Node<E: Env + Clone> {
    pub parent: usize,
    pub public_info: E::PublicInfo,
    pub my_private_info: E::PrivateInfo,
    pub terminal: bool,
    pub expanded: bool,
    pub my_action: bool,
    pub children: Vec<(E::Action, usize)>,
    pub reward: f32,
    pub num_visits: f32,
}

impl<E: Env + Clone> Node<E> {
    pub fn new_root(
        my_action: bool,
        public_info: &E::PublicInfo,
        my_private_info: &E::PrivateInfo,
    ) -> Self {
        Node {
            parent: 0,
            public_info: public_info.clone(),
            my_private_info: my_private_info.clone(),
            terminal: false,
            expanded: false,
            my_action: my_action,
            children: Vec::new(),
            num_visits: 0.0,
            reward: 0.0,
        }
    }

    pub fn new(parent_id: usize, node: &Self, action: &E::Action) -> Self {
        let mut env = node.env.clone();
        let is_over = env.step(action);
        Node {
            parent: parent_id,
            env: env,
            terminal: is_over,
            expanded: is_over,
            my_action: !node.my_action,
            children: Vec::new(),
            num_visits: 0.0,
            reward: 0.0,
        }
    }
}

pub struct MCTS<E: Env + Clone> {
    pub id: bool,
    pub root: usize,
    pub nodes: Vec<Node<E>>,
    pub rng: StdRng, // note: this is about the same performance as SmallRng or any of the XorShiftRngs that got moved to the xorshift crate
}

impl<E: Env + Clone> MCTS<E> {
    pub fn with_capacity(id: bool, capacity: usize, seed: u64) -> Self {
        let mut nodes = Vec::with_capacity(capacity);
        let root = Node::new_root(id == WHITE);
        nodes.push(root);
        Self {
            id: id,
            root: 0,
            nodes: nodes,
            rng: StdRng::seed_from_u64(seed),
        }
    }

    fn next_node_id(&self) -> usize {
        self.nodes.len()
    }

    pub fn step_action(&mut self, action: &E::Action) {
        // note: this function attempts to drop obviously unused nodes in order to reduce memory usage
        self.root = match self.nodes[self.root]
            .children
            .iter()
            .position(|(a, _)| a == action)
        {
            Some(action_index) => {
                let (a, new_root) = self.nodes[self.root].children[action_index];
                new_root
            }
            None => {
                let child_id = self.next_node_id();
                let child_node = Node::new(child_id, &self.nodes[self.root], action);
                self.nodes.push(child_node);
                child_id
            }
        };
    }

    pub fn best_action(&self) -> E::Action {
        let root = &self.nodes[self.root];

        let mut best_action_ind = 0;
        let mut best_value = -std::f32::INFINITY;

        for (i, &(_, child_id)) in root.children.iter().enumerate() {
            let child = &self.nodes[child_id];
            let value = child.reward / child.num_visits;
            if value > best_value {
                best_value = value;
                best_action_ind = i;
            }
        }

        root.children[best_action_ind].0
    }

    fn explore(&mut self) {
        let mut node_id = self.root;
        loop {
            // assert!(node_id < self.nodes.len());
            let node = &mut self.nodes[node_id];
            if node.terminal {
                let reward = node.public_info.reward();
                self.backprop(node_id, reward, 1.0);
                return;
            } else if node.expanded {
                node_id = self.select_best_child(node_id);
            } else {
                // expand all children at once
                let (total_reward, total_visits) = self.expand_all_children(node_id);

                // backprop all new children rewards back up
                self.backprop(node_id, total_reward, total_visits);

                // we've expanded one node now, 1 round of exploring done!
                return;
            }
        }
    }

    fn select_best_child(&mut self, node_id: usize) -> usize {
        // assert!(node_id < self.nodes.len());
        let node = &self.nodes[node_id];

        let visits = node.num_visits.log(2.0);

        let raw_first_child = node.children[0].1;
        let first_child = raw_first_child - self.root;
        let last_child = first_child + node.children.len();

        let best_child_ind = self.nodes[first_child..last_child]
            .iter()
            .map(|child| child.reward / child.num_visits + (2.0 * visits / child.num_visits).sqrt())
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap();

        best_child_ind + raw_first_child
    }

    fn expand_all_children(&mut self, node_id: usize) -> (f32, f32) {
        let mut node = &mut self.nodes[node_id];

        // we are adding all children at once, so this node is about to be expanded
        node.expanded = true;

        let mut total_reward = 0.0;
        let mut total_visits = 0.0;

        // TODO sample env so we can create actions
        let actions = Vec::new();

        // reserve max number of actions for children to reduce allocations
        node.children.reserve_exact(actions.len());

        // iterate through all the children!
        for action in actions {
            // create the child node and sample a reward from it
            let child_node = self.expand_single_child(node_id, action);

            // keep track of reward here so we can backprop 1 time for all the new children
            total_reward += child_node.reward;
            total_visits += 1.0;

            self.nodes.push(child_node);
        }

        (total_reward, total_visits)
    }

    fn expand_single_child(&mut self, node_id: usize, action: E::Action) -> Node<E> {
        let child_id = self.next_node_id();

        let node = &mut self.nodes[node_id];
        node.children.push((action, child_id));

        // create the child node... note we will be modifying num_visits and reward later, so mutable
        let mut child_node = Node::new(node_id, &node, &action);

        // rollout child to get initial reward
        // TODO sample rollout here
        let reward = self.rollout(child_node.env.clone());

        // store initial reward & 1 visit
        child_node.num_visits = 1.0;
        child_node.reward = reward;

        child_node
    }

    fn rollout(&mut self, mut env: E) -> f32 {
        // assert!(node_id < self.nodes.len());
        // note: checking if env.is_over() before cloning doesn't make much difference
        let mut is_over = env.is_over();
        while !is_over {
            let action = env.get_random_action(&mut self.rng);
            is_over = env.step(&action);
        }
        env.reward(self.id)
    }

    fn backprop(&mut self, leaf_node_id: usize, reward: f32, num_visits: f32) {
        let mut node_id = leaf_node_id;
        loop {
            // assert!(node_id < self.nodes.len());

            let node = &mut self.nodes[node_id];

            node.num_visits += num_visits;

            node.reward += reward;

            if node_id == self.root {
                break;
            }

            node_id = node.parent;
        }
    }

    pub fn explore_for(&mut self, millis: u128) -> (usize, u128) {
        let start = Instant::now();
        let start_n = self.nodes.len();
        while start.elapsed().as_millis() < millis {
            self.explore();
        }
        (self.nodes.len() - start_n, start.elapsed().as_millis())
    }

    pub fn explore_n(&mut self, n: usize) -> (usize, u128) {
        let start = Instant::now();
        let start_n = self.nodes.len();
        for _ in 0..n {
            self.explore();
        }
        (self.nodes.len() - start_n, start.elapsed().as_millis())
    }
}
