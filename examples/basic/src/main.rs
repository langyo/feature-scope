use feature_scope::{feature_scope, feature_scope_default};

#[feature_scope(a)]
pub fn basic_expand() {
    println!("feature_scope_a");
}

#[feature_scope_default]
pub fn basic_expand() {
    {
        println!("feature_scope_default");
    }
}

pub fn main() {
    basic_expand();
}
