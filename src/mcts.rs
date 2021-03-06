use crate::env::Env;
use crate::rand::rngs::StdRng;
use crate::rand::SeedableRng;
use std::time::Instant;

pub struct Node<E: Env + Clone> {
    pub parent: usize,                     // 8 bytes
    pub public_info: E::PublicInfo,        // 32 bytes
    pub terminal: bool,                    // 1 byte
    pub expanded: bool,                    // 1 byte
    pub my_action: bool,                   // 1 byte
    pub children: Vec<(E::Action, usize)>, // 24 bytes
    pub reward: f32,                       // 4 bytes
    pub num_visits: f32,                   // 4 bytes
}

impl<E: Env + Clone> Node<E> {
    pub fn new_root(my_action: bool) -> Self {
        Node {
            parent: 0,
            public_info: E::new(),
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
            in_symmetry: false,
        }
    }

    fn next_node_id(&self) -> usize {
        self.nodes.len() + self.root
    }

    pub fn step_action(&mut self, action: &E::Action) {
        // note: this function attempts to drop obviously unused nodes in order to reduce memory usage
        self.root = match self.nodes[self.root - self.root]
            .children
            .iter()
            .position(|(a, c, sym_a)| a == action || (sym_a.is_some() && sym_a.unwrap() == *action))
        {
            Some(action_index) => {
                let (a, new_root, _) = self.nodes[self.root - self.root].children[action_index];
                self.in_symmetry = a != *action;
                drop(self.nodes.drain(0..new_root - self.root));
                new_root
            }
            None => {
                self.in_symmetry = false;
                let child_node = Node::new(0, &self.nodes[self.root - self.root], action);
                self.nodes.clear();
                self.nodes.push(child_node);
                0
            }
        };

        self.nodes[0].parent = self.root;
    }

    pub fn best_action(&self) -> E::Action {
        let root = &self.nodes[self.root - self.root];

        let mut best_action_ind = 0;
        let mut best_value = -std::f32::INFINITY;

        for (i, &(_, child_id, _)) in root.children.iter().enumerate() {
            let child = &self.nodes[child_id - self.root];
            let value = child.reward / child.num_visits;
            if value > best_value {
                best_value = value;
                best_action_ind = i;
            }
        }

        if self.in_symmetry && root.children[best_action_ind].2.is_some() {
            root.children[best_action_ind].2.unwrap()
        } else {
            root.children[best_action_ind].0
        }
    }

    pub fn negamax(&self, depth: u8) -> E::Action {
        // TODO add alpha beta pruning to this
        let best_action_ind = self.negamax_search(self.root, depth, -1.0).0.unwrap();

        let root = &self.nodes[self.root - self.root];
        if self.in_symmetry && root.children[best_action_ind].2.is_some() {
            root.children[best_action_ind].2.unwrap()
        } else {
            root.children[best_action_ind].0
        }
    }

    fn negamax_search(&self, node_id: usize, depth: u8, color: f32) -> (Option<usize>, f32) {
        let node = &self.nodes[node_id - self.root];
        if depth == 0 || node.terminal || !node.expanded {
            return (None, color * node.reward / node.num_visits);
        }

        let mut best_value = -std::f32::INFINITY;
        let mut best_action_ind = 0;
        for (i, &(_, child_id, _)) in node.children.iter().enumerate() {
            let (_, v) = self.negamax_search(child_id, depth - 1, -color);
            if -v > best_value {
                best_value = -v;
                best_action_ind = i;
            }
        }

        (Some(best_action_ind), best_value)
    }

    fn explore(&mut self) {
        let mut node_id = self.root;
        loop {
            // assert!(node_id < self.nodes.len());
            let node = &mut self.nodes[node_id - self.root];
            if node.terminal {
                let reward = node.env.reward(self.id);
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
        let node = &self.nodes[node_id - self.root];

        let mut best_child = 0;
        let mut best_value = -std::f32::INFINITY;

        let visits = node.num_visits.log(2.0);

        // TODO try slices again after symmetry update... lower branching factor may result in better cache usage & deeper tree means this function gets called more
        // note: using a slide of self.nodes[first_child..last_child] doesn't result in a performance increase
        for &(_, child_id, _) in node.children.iter() {
            let child = &self.nodes[child_id - self.root];

            let value = child.reward / child.num_visits + (2.0 * visits / child.num_visits).sqrt();

            if value > best_value {
                best_value = value;
                best_child = child_id;
            }
        }

        best_child
    }

    fn expand_all_children(&mut self, node_id: usize) -> (f32, f32) {
        let mut node = &mut self.nodes[node_id - self.root];

        // reserve max number of actions for children to reduce allocations
        node.children.reserve_exact(48);

        // we are adding all children at once, so this node is about to be expanded
        node.expanded = true;

        let mut total_reward = 0.0;
        let mut total_visits = 0.0;

        let actions = node.env.actions();
        let mut closed = Vec::with_capacity(actions.len());

        // iterate through all the children!
        for &action in actions.iter() {
            // skip any actions that we've already added (possible because of symmetric actions)
            if closed.contains(&action) {
                continue;
            }

            // don't look at this action anymore
            closed.push(action);

            // add the symmetric action to closed if its a valid action... and keep track of it in children
            let symmetrical_action = E::symmetry_of(&action);
            let symmetry = if actions.contains(&symmetrical_action) {
                closed.push(symmetrical_action);
                Some(symmetrical_action)
            } else {
                None
            };

            // create the child node and sample a reward from it
            let child_node = self.expand_single_child(node_id, action, symmetry);

            // keep track of reward here so we can backprop 1 time for all the new children
            total_reward += child_node.reward;
            total_visits += 1.0;

            self.nodes.push(child_node);
        }

        (total_reward, total_visits)
    }

    fn expand_single_child(
        &mut self,
        node_id: usize,
        action: E::Action,
        symmetry: Option<E::Action>,
    ) -> Node<E> {
        let child_id = self.next_node_id();

        let node = &mut self.nodes[node_id - self.root];
        node.children.push((action, child_id, symmetry));

        // create the child node... note we will be modifying num_visits and reward later, so mutable
        let mut child_node = Node::new(node_id, &node, &action);

        // rollout child to get initial reward
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

            let node = &mut self.nodes[node_id - self.root];

            node.num_visits += num_visits;

            // TODO multiply reward by -1 instead of this if every time
            // note this is reversed because its actually the previous node's action that this node's reward is associated with
            node.reward += if !node.my_action { reward } else { -reward };

            if node_id == self.root {
                break;
            }

            node_id = node.parent;
        }
    }

    pub fn explore_for(&mut self, millis: u128) -> (usize, u128) {
        let start = Instant::now();
        let mut steps = 0;
        while start.elapsed().as_millis() < millis {
            self.explore();
            steps += 1;
        }
        (steps, start.elapsed().as_millis())
    }

    pub fn explore_n(&mut self, n: usize) -> (usize, u128) {
        let start = Instant::now();
        let start_n = self.nodes.len();
        let target_n = start_n + n;
        for _ in 0..n {
            self.explore();
            if self.nodes.len() >= target_n {
                break;
            }
        }
        (self.nodes.len() - start_n, start.elapsed().as_millis())
    }
}
