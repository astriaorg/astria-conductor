pub struct ValidatorSet {
    pub block_height: String,
    pub validators: Vec<Validator>,
}

pub struct Validator {
    pub address: String,
    pub pub_key: String,
    pub voting_power: String,
    pub proposer_priority: String,
}