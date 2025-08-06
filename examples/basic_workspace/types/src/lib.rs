#[feature_scope::feature_scope(a)]
pub fn test() {
    println!("a type");
}

#[feature_scope::feature_scope(b)]
pub fn test() {
    println!("b type");
}

#[feature_scope::feature_scope_default]
pub fn test() {
    println!("default type");
}
