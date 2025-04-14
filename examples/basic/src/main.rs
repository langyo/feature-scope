use feature_scope::feature_scope;

#[feature_scope(a)]
pub fn basic_expand() {
    {
        println!("feature_scope_a");
    }
}

// #[feature_scope(not(a))]
// pub fn basic_expand() {
//     {
//         println!("feature_scope_not_a");
//     }
// }

pub fn main() {
    basic_expand();
}
