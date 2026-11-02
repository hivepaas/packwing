//! Prints the policies this computer sets, without the window: for trying the
//! system's reader where the app is not built.

fn main() {
    print!("{}", packwing_policy::current().report());
}
