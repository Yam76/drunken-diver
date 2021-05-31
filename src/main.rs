use drunken_diver::{Route, Dive};

fn main() {

    let s = b"f4bf9f7fcbedaba0392f108c59d8f4a38b3838efb64877380171b54475c2ade8".into_iter();
    println!("{}", Route::<16>::from(Dive::from(s.map(|x| *x))));

    let s = b"2062f80093066633876b542212c496501a5e79523cc4ea9b28667dff065afd8f".into_iter();
    println!("{}", Route::<16>::from(Dive::from(s.map(|x| *x))));

    let s = b"f4bf9f7fcbedaba0392f108c59d8f4a38b3838efb64877380171b54475c2ade8".into_iter();
    println!("{}", Route::<32>::from(Dive::from(s.map(|x| *x))));

    let s = b"2062f80093066633876b542212c496501a5e79523cc4ea9b28667dff065afd8f".into_iter();
    println!("{}", Route::<32>::from(Dive::from(s.map(|x| *x))));
}
